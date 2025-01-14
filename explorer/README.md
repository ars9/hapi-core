# hapi-explorer

HAPI explorer multichain backend

---

## Setup

In order to be able to use this client, you will need to use Postgres database.
It is required to set explorer configuration before start. Define config file in CONFIG_PATH env variable.
Configuration file must contain fields:

```toml
log_level                           # Log level for the application layer, default: info
is_json_logging                     # Whether to use JSON logging, default: true
enable_metrics                      # Whether to enable metrics, default: true
listener                            # Address for the listener server
database_url                        # The database url
```

Also add secret from jwt to configuration file, defined in SECRET_PATH env variable:

```toml
jwt_secret                          # Secret from JWT
```

## Usage

To run cli in the repo root:

```sh
cargo run
```

or to run directly the binary:

```sh
cd ./target/debug && hapi-explorer
```

---

HAPI explorer cli includes the following commands:

| Command        | Description                                              |
| -------------- | -------------------------------------------------------- |
| server         | Runs HAPI Explorer multichain backend                    |
| migrate        | Contains a set of subcommands for managing migrations    |
| network        | Contains a set of subcommands for network management     |
| create-indexer | Creates indexer for the given network                    |
| help           | Display available commands                               |

### Running explorer server

To run HAPI Explorer multichain backend that will be handling client GraphQL requests:

```sh
hapi-explorer server
```

### Manage explorer migrations

To manage migrations for HAPI Explorer multichain backend run:

```sh
hapi-explorer migrate
```

with subcommands:

| Subcommand        | Description                                                    |
| ----------------- | -------------------------------------------------------------- |
| fresh             | Drop all tables from the database, then reapply all migrations |
| refresh           | Rollback all applied migrations, then reapply all migrations   |
| reset             | Rollback all applied migrations                                |
| status            | Check the status of all migrations                             |
| up -n `<COUNT>`   | Apply pending migrations                                       |
| down -n `<COUNT>` | Rollback applied migrations                                    |

### Manage networks

- To create new network:

  ```sh
  hapi-explorer network create --id <ID> --name <NAME> --backend <BACKEND> --authority <AUTHORITY> --stake-token <STAKE_TOKEN> --chain-id <CHAIN_ID>
  ```

  (where chain-id is optional)

- To update existing network

  ```sh
  hapi-explorer network update [OPTIONS] --id <ID> --name <NAME> --authority <AUTHORITY> --stake-token <STAKE_TOKEN>
  ```

  (where name, authority and stake-token is optional)

---

Network options:

| Option        | Description                            |
| ------------- | -------------------------------------- |
| --id          | Network string identifier              |
| --name        | Network display name                   |
| --backend     | Network backend type: evm solana, near |
| --authority   | Network authority address              |
| --stake-token | Stake token contract address           |
| --chain-id    | Optional chain id                      |

### Creating a new indexer

This command will create a new indexer for the given network. The indexer will be added to the `indexers` table in the database.
Program, will log the indexer's jwt token to the console. This token should be used to create a new indexer client.

```sh
cargo run create-indexer --network=near
```

To use custom secret_phrase, set the `SECRET_PATH` environment variable to the path of the file containing the secret phrase. It must be .toml file with the following format:

```toml
jwt_secret="secret_phrase"
```

## Running tests

Currently due to the peculiarities of test execution, the launch should take place in one thread:

```sh
cargo test -- --test-threads=1
```

## License

HAPI explorer is distributed under the terms of the MIT license.
