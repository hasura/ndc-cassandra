//! # Connector Definition
//!
//! Provides HTTP server paths for required NDC functions. Connecting
//! the request to the underlying code and providing the result.
//!
use std::collections::BTreeMap;
use std::{env, fs};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use log::{info, error};
use async_trait::async_trait;
use dotenv;
use jni::objects::GlobalRef;
use ndc_models as models;
use ndc_models::{ArgumentName, Capabilities, CollectionName, Relationship, RelationshipName, VariableName};
use ndc_sdk::connector::{
    Connector, ConnectorSetup, ExplainError, FetchMetricsError, HealthError, InitializationError,
    MutationError, ParseError, QueryError, SchemaError,
};
use ndc_sdk::json_response::JsonResponse;
use serde_json::Value;
use tracing::{info_span, Level, span, Span};
use tracing::Instrument;

use crate::capabilities::calcite_capabilities;
use ndc_calcite_schema::jvm::{get_jvm, init_jvm};
use ndc_calcite_schema::calcite::Model;
use ndc_calcite_schema::models::get_models;
use ndc_calcite_schema::schema::get_schema as retrieve_schema;
use ndc_calcite_schema::version5::ParsedConfiguration;
use ndc_calcite_values::is_running_in_container::is_running_in_container;
use ndc_calcite_values::values::{CONFIGURATION_FILENAME, DEV_CONFIG_FILE_NAME};
use crate::{calcite, query};
use crate::query::QueryParams;

#[derive(Clone, Default, Debug)]
pub struct Calcite {}

#[derive(Clone, Debug)]
pub struct CalciteState {
    pub calcite_ref: GlobalRef,
}

#[tracing::instrument(skip(config, coll, args, coll_rel, query, vars, state, explain))]
fn execute_query_with_variables(
    config: &ParsedConfiguration,
    coll: &CollectionName,
    args: &BTreeMap<ArgumentName, models::RelationshipArgument>,
    coll_rel: &BTreeMap<RelationshipName, Relationship>,
    query: &models::Query,
    vars: &BTreeMap<VariableName, Value>,
    state: &CalciteState,
    explain: &bool
) -> Result<models::RowSet, QueryError> {
    query::orchestrate_query(QueryParams { config, coll, coll_rel, args, query, vars, state, explain})
}

const CONFIG_ERROR_MSG: &str = "Could not find model file.";

fn has_yaml_extension(filename: &str) -> bool {
    let path = Path::new(filename);
    match path.extension().and_then(OsStr::to_str) {
        Some("yml") | Some("yaml") => true,
        _ => false,
    }
}

#[async_trait]
impl ConnectorSetup for Calcite {
    type Connector = Self;

    #[tracing::instrument(skip(configuration_dir))]
    async fn parse_configuration(
        &self,
        configuration_dir: impl AsRef<Path> + Send,
    ) -> Result<<Self as Connector>::Configuration, ParseError> {

        let span = span!(Level::INFO, "parse_configuration");
        dotenv::dotenv().ok();

        fn get_config_file_path(configuration_dir: impl AsRef<Path> + Send) -> PathBuf {
            if is_running_in_container() {
                configuration_dir.as_ref().join(CONFIGURATION_FILENAME)
            } else {
                configuration_dir.as_ref().join(DEV_CONFIG_FILE_NAME)
            }
        }

        fn configure_path(span: Span, configuration_dir: &(impl AsRef<Path> + Send)) {
            println!(
                "Configuration file path: {:?}",
                configuration_dir.as_ref().display()
            );
            span.record("configuration_dir", format!("{:?}", configuration_dir.as_ref().display()));
        }

        fn parse_json<T: Connector<Configuration = ParsedConfiguration>>(json_str: String) -> Result<T::Configuration, ParseError> {
            let mut json_object: ParsedConfiguration = serde_json::from_str(&json_str)
                .map_err(|err| ParseError::Other(Box::from(err.to_string())))?;

            update_model(&mut json_object)?;
            update_metadata(&mut json_object);

            Ok(json_object)
        }

        fn update_model(json_object: &mut ParsedConfiguration) -> Result<(), ParseError> {
            let model_file_path = json_object
                .model_file_path
                .clone()
                .or_else(|| env::var("MODEL_FILE").ok())
                .ok_or(ParseError::Other(Box::from(CONFIG_ERROR_MSG)))?;

            let models = fs::read_to_string(model_file_path.clone())?;

            if has_yaml_extension(&model_file_path.clone()) {
                let model_object: Model = serde_yaml::from_str(&models)
                    .map_err(|err| ParseError::Other(Box::from(err.to_string())))?;
                json_object.model = Some(model_object);
            } else {
                let model_object: Model = serde_json::from_str(&models)
                    .map_err(|err| ParseError::Other(Box::from(err.to_string())))?;
                json_object.model = Some(model_object);
            }

            Ok(())
        }

        fn update_metadata(json_object: &mut ParsedConfiguration) {
            if json_object.metadata.is_none() {
                let state = init_state(&json_object).expect("TODO: panic message");
                json_object.metadata = Some(get_models(&state.calcite_ref));
                println!("metadata: {:?}", serde_json::to_string_pretty(&json_object.metadata));
            }
        }

        configure_path(span, &configuration_dir);
        match fs::read_to_string(get_config_file_path(configuration_dir)) {
            Ok(file_content) => parse_json::<Self>(file_content),
            Err(err) => Err(ParseError::Other(Box::from(err.to_string()))),
        }
    }

    async fn try_init_state(
        &self,
        configuration: &<Self as Connector>::Configuration,
        _metrics: &mut prometheus::Registry,
    ) -> Result<<Self as Connector>::State, InitializationError> {
        init_state(configuration)
    }

}

#[async_trait]
impl Connector for Calcite {
    type Configuration = ParsedConfiguration;
    type State = CalciteState;

    fn fetch_metrics(
        _configuration: &Self::Configuration,
        _state: &Self::State,
    ) -> Result<(), FetchMetricsError> {
        Ok(())
    }

    async fn health_check(
        _configuration: &Self::Configuration,
        _state: &Self::State,
    ) -> Result<(), HealthError> {
        Ok(())
    }

    async fn get_capabilities() -> Capabilities {
        calcite_capabilities().capabilities
    }

    async fn get_schema(
        configuration: &Self::Configuration,
    ) -> Result<JsonResponse<models::SchemaResponse>, SchemaError> {
        async {
            info_span!("inside tracing Calcite");
        }
            .instrument(info_span!("tracing Calcite"))
            .await;
        dotenv::dotenv().ok();
        let calcite;
        let calcite_ref;
        {
            let java_vm = get_jvm().lock().unwrap();
            let mut env = java_vm.attach_current_thread_as_daemon().unwrap();
            calcite = calcite::create_calcite_query_engine(configuration, &mut env);
            let env = java_vm.attach_current_thread_as_daemon().unwrap();
            calcite_ref = env.new_global_ref(calcite).unwrap();
        }

        let schema = retrieve_schema(configuration, calcite_ref.clone());
        match schema {
            Ok(schema) => Ok(JsonResponse::from(schema)),
            Err(e) => Err(SchemaError::Other(e.to_string().into(), Value::Null)),
        }
    }

    async fn query_explain(
        configuration: &Self::Configuration,
        state: &Self::State,
        request: models::QueryRequest,
    ) -> Result<JsonResponse<models::ExplainResponse>, ExplainError> {
        let variable_sets = request.variables.unwrap_or(vec![BTreeMap::new()]);
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        let input_map: BTreeMap<ArgumentName, models::Argument> = request.arguments.clone();
        let relationship_arguments : BTreeMap<ArgumentName, models::RelationshipArgument> =
            input_map.iter()
                .map(|(key, value)|
                (key.clone(), convert_to_relationship_argument(value))
                )
                .collect();
        for variables in &variable_sets {
            let row_set = execute_query_with_variables(
                configuration,
                &request.collection,
                &relationship_arguments,
                &request.collection_relationships,
                &request.query,
                variables,
                &state,
                &true
            ).map_err(|error| ExplainError::Other(Box::new(error), Value::Null))?;
            match row_set.aggregates {
                None => {}
                Some(map_index) => {
                    let map_btree: BTreeMap<String, String> = map_index.iter()
                        .map(|(key, value)| (key.clone().to_string(), value.to_string()))
                        .collect();
                    map.extend(map_btree);
                }
            };
            match row_set.rows {
                None => {}
                Some(r) => {
                    for map_index in r {
                        let map_btree: BTreeMap<String, String> = map_index.iter()
                            .map(|(key, value)| (key.clone().to_string(), value.0.to_string()))
                            .collect();
                        map.extend(map_btree);
                    }
                }
            }
        }
        let explain_response = models::ExplainResponse {
            details: map,
        };
        Ok(JsonResponse::from(explain_response))
    }

    async fn mutation_explain(
        _configuration: &Self::Configuration,
        _state: &Self::State,
        _request: models::MutationRequest,
    ) -> Result<JsonResponse<models::ExplainResponse>, ExplainError> {
        todo!()
    }

    async fn mutation(
        _configuration: &Self::Configuration,
        _state: &Self::State,
        _request: models::MutationRequest,
    ) -> Result<JsonResponse<models::MutationResponse>, MutationError> {
        todo!()
    }

    async fn query(
        configuration: &Self::Configuration,
        state: &Self::State,
        request: models::QueryRequest,
    ) -> Result<JsonResponse<models::QueryResponse>, QueryError> {
        // println!("{:?}", serde_json::to_string_pretty(&request));
        let variable_sets = request.variables.unwrap_or(vec![BTreeMap::new()]);
        let mut row_sets = vec![];
        let input_map: BTreeMap<ArgumentName, models::Argument> = request.arguments.clone();
        let relationship_arguments : BTreeMap<ArgumentName, models::RelationshipArgument> =
            input_map.iter()
                .map(|(key, value)|
                // Assuming we have a function `convert_to_relationship_argument`
                // that takes an argument and returns a relationship argument
                (key.clone(), convert_to_relationship_argument(value))
                )
                .collect();
        for variables in &variable_sets {
            let row_set = match execute_query_with_variables(
                configuration,
                &request.collection,
                &relationship_arguments,
                &request.collection_relationships,
                &request.query,
                variables,
                &state,
                &false
            ) {
                Ok(row_set) => {
                    info!("execute_query_with_variables was successful");
                    row_set
                },
                Err(e) => {
                    error!("Error executing query: {:?}", e);
                    return Err(e.into());
                },
            };
            // println!("Get row set");
            // println!("{:?}", serde_json::to_string_pretty(&row_set));
            row_sets.push(row_set);
            // println!("Pushed row set");
        }
        // println!("Returning row sets");
        Ok(models::QueryResponse(row_sets).into())
    }
}

fn convert_to_relationship_argument(p0: &models::Argument) -> models::RelationshipArgument {
    match p0 {
        models::Argument::Variable { name } => models::RelationshipArgument::Variable { name: name.clone() },
        models::Argument::Literal { value } => models::RelationshipArgument::Literal { value: value.clone() }
    }
}

fn init_state(
    configuration: &ParsedConfiguration,
) -> Result<CalciteState, InitializationError> {
    dotenv::dotenv().ok();
    init_jvm(&ndc_calcite_schema::configuration::ParsedConfiguration::Version5(configuration.clone()));
    let java_vm = get_jvm().lock().unwrap();
        let calcite;
        let calcite_ref;
        {
            let mut env = java_vm.attach_current_thread_as_daemon().unwrap();
            calcite = calcite::create_calcite_query_engine(configuration, &mut env);
            let env = java_vm.attach_current_thread_as_daemon().unwrap();
            calcite_ref = env.new_global_ref(calcite).unwrap();
        }
        Ok(CalciteState { calcite_ref })
}
#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::path::PathBuf;

    use axum_test_helper::TestClient;
    use http::StatusCode;

    use super::*;

    #[tokio::test]
    async fn capabilities_match_ndc_spec_version() -> Result<(), Box<dyn Error + Send + Sync>> {
        let state =
            ndc_sdk::default_main::init_server_state(Calcite::default(), PathBuf::new()).await?;
        let app = ndc_sdk::default_main::create_router::<Calcite>(state, None);

        let client = TestClient::new(app);
        let response = client.get("/capabilities").send().await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: ndc_models::CapabilitiesResponse = response.json().await;
        // ideally we would get this version from `ndc_models::VERSION`
        assert_eq!(body.version, "0.1.4");
        Ok(())
    }
}
