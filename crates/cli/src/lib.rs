//! The interpretation of the commands that the CLI can handle.
//!
//! The CLI can do a few things. This provides a central point where those things are routed and
//! then done, making it easier to test this crate deterministically.

use std::path::{PathBuf};

use clap::Subcommand;
use include_dir::{DirEntry, include_dir};
use include_dir::Dir;
use tokio::fs;

use ndc_calcite_schema::configuration::{has_configuration, introspect, parse_configuration, ParsedConfiguration, upgrade_to_latest_version, write_parsed_configuration};
use ndc_calcite_schema::environment::Environment;
use ndc_calcite_schema::jvm::init_jvm;
use ndc_calcite_schema::version5::CalciteRefSingleton;
use ndc_calcite_values::is_running_in_container::is_running_in_container;
use ndc_calcite_values::values::{DOCKER_CONNECTOR_DIR, DOCKER_CONNECTOR_RW, DOCKER_IMAGE_NAME, UNABLE_TO_WRITE_TO_FILE};

mod metadata;

const UPDATE_ATTEMPTS: u8 = 3;

/// The various contextual bits and bobs we need to run.
pub struct Context<Env: Environment> {
    pub context_path: PathBuf,
    pub environment: Env,
    pub release_version: Option<&'static str>,
}

/// The command invoked by the user.
#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Initialize a configuration in the current (empty) directory.
    Initialize {
        #[arg(long)]
        /// Whether to create the hasura connector metadata.
        with_metadata: bool,
    },
    /// Update the configuration by introspecting the database, using the configuration options.
    Update,
    /// Upgrade the configuration to the latest version. This does not involve the database.
    Upgrade {
        #[arg(long)]
        dir_from: PathBuf,
        #[arg(long)]
        dir_to: PathBuf,
    },
}

/// The set of errors that can go wrong _in addition to_ generic I/O or parsing errors.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("directory is not empty")]
    DirectoryIsNotEmpty,
}

/// Run a command in a given directory.
#[tracing::instrument(skip(context,calcite_ref_singleton))]
pub async fn run(command: Command, context: Context<impl Environment>, calcite_ref_singleton: CalciteRefSingleton) -> anyhow::Result<()> {
    match command {
        Command::Initialize { with_metadata } => initialize(with_metadata, &context).await?,
        Command::Update => update(context, &calcite_ref_singleton).await?,
        Command::Upgrade { dir_from, dir_to } => upgrade(dir_from, dir_to).await?,
    };
    Ok(())
}

const MODELS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

/// Initialize an empty directory with an empty connector configuration.
///
/// An empty configuration contains default settings and options, and is expected to be filled with
/// information such as the database connection string by the user, and later on metadata
/// information via introspection.
///
/// Optionally, this can also create the connector metadata, which is used by the Hasura CLI to
/// automatically work with this CLI as a plugin.

#[tracing::instrument(skip(context))]
async fn initialize(with_metadata: bool, context: &Context<impl Environment>) -> anyhow::Result<()> {
    let docker_config_path = &PathBuf::from(DOCKER_CONNECTOR_RW);
    let config_path = if is_running_in_container() {
        docker_config_path
    } else {
        &context.context_path
    };
    if has_configuration(config_path) {
        Err(Error::DirectoryIsNotEmpty)?;
    }

    write_parsed_configuration(ParsedConfiguration::initial(), config_path, ).await?;

    for entry in MODELS_DIR.find("**/*").unwrap() {
        match entry {
            DirEntry::Dir(dir) => {
                let path = dir.path();
                fs::create_dir((config_path).join(path)).await?
            }
            DirEntry::File(file) => {
                let path = file.path();
                let contents = file.contents();
                std::fs::write((config_path).join(path), contents).expect(UNABLE_TO_WRITE_TO_FILE);
            }
        }
    }

    // if requested, create the metadata
    if with_metadata {
        let metadata_dir = config_path.join(".hasura-connector");
        fs::create_dir(&metadata_dir).await?;
        let metadata_file = metadata_dir.join("connector-metadata.yaml");
        let docker_image = format!(
            "{}:{}",
            DOCKER_IMAGE_NAME,
            context.release_version.unwrap_or("latest")
        );
        let update_command = format!("docker run --entry-point ndc-calcite-cli -e HASURA_PLUGIN_CONNECTOR_CONTEXT_PATH -v ${{HASURA_PLUGIN_CONNECTOR_CONTEXT_PATH}}:{} -v ${{HASURA_PLUGIN_CONNECTOR_CONTEXT_PATH}}:{}:ro {} update", DOCKER_CONNECTOR_DIR, DOCKER_CONNECTOR_RW, docker_image);
        let metadata = metadata::ConnectorMetadataDefinition {
            packaging_definition: metadata::PackagingDefinition::PrebuiltDockerImage(
                metadata::PrebuiltDockerImagePackaging { docker_image },
            ),
            supported_environment_variables: vec![metadata::EnvironmentVariableDefinition {
                name: "MODEL_FILE".to_string(),
                description: "The calcite connection model file path".to_string(),
                default_value: Some(format!("{}/models/model.json", DOCKER_CONNECTOR_DIR).to_string()),
            }],
            commands: metadata::Commands {
                update: Some(update_command.to_string()),
                watch: None,
            },
            cli_plugin: None,
            docker_compose_watch: vec![metadata::DockerComposeWatchItem {
                path: "./".to_string(),
                target: Some(DOCKER_CONNECTOR_RW.to_string()),
                action: metadata::DockerComposeWatchAction::SyncAndRestart,
                ignore: vec![],
            }],
        };

        fs::write(metadata_file, serde_yaml::to_string(&metadata)?).await?;
    }

    Ok(())
}

/// Update the configuration in the current directory by introspecting the database.
///
/// If the directory is empty - it will initialize with the core files first.
#[tracing::instrument(skip(context,calcite_ref_singleton))]
async fn update(context: Context<impl Environment>, calcite_ref_singleton: &CalciteRefSingleton) -> anyhow::Result<()> {
    let docker_config_path = &PathBuf::from(DOCKER_CONNECTOR_RW);
    let config_path = if is_running_in_container() {
        docker_config_path
    } else {
        &context.context_path
    };
    if !has_configuration(config_path) {
        initialize(true, &context).await?
    }

    // It is possible to change the file in the middle of introspection.
    // We want to detect this scenario and retry, or fail if we are unable to.
    // We do that with a few attempts.
    for _attempt in 1..=UPDATE_ATTEMPTS {
        let existing_configuration =
            parse_configuration(config_path).await?;
        init_jvm(&existing_configuration);

        let output =
            introspect(existing_configuration.clone(), config_path, &context.environment, calcite_ref_singleton).await?;

        // Check that the input file did not change since we started introspecting,
        let input_again_before_write =
            parse_configuration(config_path).await?;

        // and skip this attempt if it has.
        if input_again_before_write == existing_configuration {
            // In order to be sure to capture default values absent in the initial input we have to
            // always write out the updated configuration.
            write_parsed_configuration(output, config_path).await?;
            return Ok(());
        }

        // If we have reached here, the input file changed before writing.
    }

    // We ran out of attempts.
    Err(anyhow::anyhow!(
        "Cannot override configuration: input changed before write."
    ))
}

/// Upgrade the configuration in a directory by trying to read it and then write it back
/// out to a different directory.
///
#[tracing::instrument(skip(dir_from, dir_to))]
async fn upgrade(dir_from: PathBuf, dir_to: PathBuf) -> anyhow::Result<()> {
    let old_configuration = parse_configuration(dir_from).await?;
    let upgraded_configuration = upgrade_to_latest_version(old_configuration);
    write_parsed_configuration(upgraded_configuration, dir_to).await?;

    eprintln!("Upgrade completed successfully. You may need to also run 'update'.");

    Ok(())
}
