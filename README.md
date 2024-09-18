# NDC Cassandra

This repository contains an adapter that is metadata configurable to support approximately 40 data sources.

Approximately 15 files-based data sources, and 25 JDBC based data sources.

## Temporary Instructions - For Getting Started as a Developer with this repo.

### Clone the repo && the subrepo

This adapter is based on a forked version of Cassandra (the sub-repo)

```shell
 git clone https://github.com/hasura/ndc-cassandra.git
 cd ndc-cassandra
 git checkout main
```

Note - this is somewhat simplified - because everything is in the "main" branch. I'll let you research how to manage
the primary and sub-branch on your own!

### Build the Java project

The project will require jdk 21 and maven. You need to have those installed first.

This is the JNI for cassandra. It handles the Cassandra to Rust handoff.

You can build it like this.
```shell
cd cassandra-rs-jni
chmod +x build.sh
./build.sh
```

This will build the Java jars that the Rust project (at the root of this mono-repo) requires.

### Build the Connector and CLI Plugin

```shell
cd ..
cargo build --bin ndc-cassandra --bin ndc-cassandra-cli
```

### Test the file adapter

Note - the test requires "metadata" to be added to configuration.json.
It should do it on first pass, but for now - you may to run it once to populate.
Then run it again to actually perform the test.

```shell
chmod +x test.sh
./test.sh # populate metadata
./test.sh # run the tests
```
### Build the docker image

```shell
chmod +x build-local.sh
./build-local.sh
```

### Create a supergraph

```shell
ddn supergraph init test-connector
cd test-connector
```

### Create a connector under default subgraph "app"
```shell
mkdir ./app/connector
ddn connector-link add cassandra --configure-connector-token secret --configure-host http://local.hasura.dev:8081 --subgraph app/subgraph.yaml --target-env-file .env
```

### Add metadata to the connector

This script is one-and-done, you can't redo without resetting back to prior state.
You might consider, committing before running this, to facilitate a rollback.
```shell
chmod +x ../cli.sh
../cli.sh ./app/connector/cassandra 8081 secret
```

### Optional Revise Cassandra Adapter

This will setup a SQLite connector. If you want to change the connector DO IT NOW. Go to `app/connector/cassandra/models/model.json` and revise the schema(s).
Look at the sample models for ideas, or, get more details from [Apache Cassandra](https://cassandra.apache.org/docs/adapter.html).

```shell
chmod +x ../cli-update-model.sh
../cli-update-model.sh ./app/connector/cassandra
```

### Start supergraph

This is to facilitate the introspection. Introspection will not work offline
with `ddn connect-link add-all`, without the connector being in connector hub.
(That's a guess, since I can't prove it.)

```shell
HASURA_DDN_PAT=$(ddn auth print-pat) docker compose --env-file .env up --build --watch
```

### Introspect

```shell
ddn connector-link update cassandra --add-all-resources --subgraph app/subgraph.yaml
```

### Build supergraph

```shell
ddn supergraph build local
```

### View in console

[Click here to launch Console View](https://console.hasura.io/local/graphql?url=http://localhost:3000)

### Execute a query

```graphql
query MyQuery {
  albums(limit: 10) {
    title
    artist {
      name
    }
  }
}
```

And you should see this:

```json
{
  "data": {
    "albums": [
      {
        "title": "For Those About To Rock We Salute You",
        "artist": {
          "name": "AC/DC"
        }
      },
      {
        "title": "Balls to the Wall",
        "artist": {
          "name": "Accept"
        }
      },
      {
        "title": "Restless and Wild",
        "artist": {
          "name": "Accept"
        }
      },
      {
        "title": "Let There Be Rock",
        "artist": {
          "name": "AC/DC"
        }
      },
      {
        "title": "Big Ones",
        "artist": {
          "name": "Aerosmith"
        }
      },
      {
        "title": "Jagged Little Pill",
        "artist": {
          "name": "Alanis Morissette"
        }
      },
      {
        "title": "Facelift",
        "artist": {
          "name": "Alice In Chains"
        }
      },
      {
        "title": "Warner 25 Anos",
        "artist": {
          "name": "Antônio Carlos Jobim"
        }
      },
      {
        "title": "Plays Metallica By Four Cellos",
        "artist": {
          "name": "Apocalyptica"
        }
      },
      {
        "title": "Audioslave",
        "artist": {
          "name": "Audioslave"
        }
      }
    ]
  }
}
```
