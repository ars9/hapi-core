use super::{create_jwt, get_test_data, RequestSender, TestData};

use {
    hapi_core::{client::events::EventName, HapiCoreNetwork},
    hapi_explorer::{
        application::Application,
        configuration::Configuration,
        entity::{address, asset, case, network::Model as NetworkModel, reporter},
        observability::setup_tracing,
    },
    hapi_indexer::{PushData, PushPayload},
    sea_orm::{DatabaseConnection, EntityTrait},
    std::{env, sync::Arc},
    tokio::{
        net::TcpListener,
        spawn,
        sync::Notify,
        task::JoinHandle,
        time::{sleep, Duration},
    },
};

pub const WAITING_INTERVAL: u64 = 100;
pub const MIGRATION_COUNT: u32 = 10;
pub const METRICS_ENV_VAR: &str = "ENABLE_METRICS";
const TRACING_ENV_VAR: &str = "ENABLE_TRACING";

pub struct TestNetwork {
    pub network: HapiCoreNetwork,
    pub model: NetworkModel,
    pub token: String,
}

pub struct TestApp {
    pub server_addr: String,
    pub db_connection: DatabaseConnection,
    pub networks: Vec<TestNetwork>,
    pub server_handle: Option<JoinHandle<()>>,
    stop_signal: Arc<Notify>,
}

pub trait FromTestPayload {
    fn from_payload(payload: &PushPayload, network_id: &str) -> Self;
}

impl TestApp {
    pub async fn start() -> Self {
        if env::var(TRACING_ENV_VAR).unwrap_or_default().eq("1") {
            if let Err(e) = setup_tracing("debug", false) {
                println!("Failed to setup tracing: {}", e);
            }
        }

        let configuration = generate_configuration();

        let mut app = Application::from_configuration(configuration.clone())
            .await
            .expect("Failed to build app");

        let db_connection = TestApp::prepare_database(&app).await;
        let networks = Self::prepare_networks(&app).await;
        app.socket = Some(
            TcpListener::bind(configuration.listener.clone())
                .await
                .expect("Failed to bind to address")
                .local_addr()
                .expect("Failed to get local address"),
        );
        let port = app.port().expect("Failed to get port");

        let stop_signal = Arc::new(Notify::new());
        let receiver = stop_signal.clone();

        // Spawn a background task
        let server_handle = spawn(async move {
            app.run_server().await.expect("Failed to run server");
            receiver.notified().await;
            app.shutdown().await.expect("Failed to shutdown app");
        });

        sleep(Duration::from_millis(WAITING_INTERVAL)).await;

        TestApp {
            server_addr: format!("http://127.0.0.1:{}", port),
            db_connection,
            server_handle: Some(server_handle),
            networks,
            stop_signal,
        }
    }

    pub async fn shutdown(&mut self) {
        self.stop_signal.notify_one();

        if let Some(handle) = self.server_handle.take() {
            handle.await.expect("Failed to shutdown server");
        };
    }

    pub async fn prepare_database(app: &Application) -> DatabaseConnection {
        let db_connection = app.state.database_conn.clone();

        app.migrate(Some(sea_orm_cli::MigrateSubcommands::Down {
            num: MIGRATION_COUNT,
        }))
        .await
        .expect("Failed to migrate down");

        app.migrate(None).await.expect("Failed to migrate up");

        db_connection
    }

    async fn prepare_networks(app: &Application) -> Vec<TestNetwork> {
        let networks = vec![
            HapiCoreNetwork::Ethereum,
            HapiCoreNetwork::Solana,
            HapiCoreNetwork::Near,
            HapiCoreNetwork::Sepolia,
        ];

        let mut res = vec![];

        for network in &networks {
            let id = network.to_string();
            let name = network.to_string();
            let backend = network.clone().into();
            let chain_id = Some(format!("{id}_chain_id"));
            let authority = "test_authority".to_string();
            let stake_token = "test_stake_token".to_string();

            app.create_network(
                id.clone(),
                name.clone(),
                backend,
                chain_id.clone(),
                authority.clone(),
                stake_token.clone(),
            )
            .await
            .expect("Failed to create network");

            let token = app
                .create_indexer(backend.to_owned().into(), chain_id.clone())
                .await
                .expect("Failed to create indexer");

            let data = TestNetwork {
                network: network.clone(),
                model: NetworkModel {
                    id,
                    name,
                    backend,
                    chain_id,
                    authority,
                    stake_token,
                    created_at: chrono::Utc::now().naive_utc(),
                    updated_at: chrono::Utc::now().naive_utc(),
                },
                token,
            };

            res.push(data);
        }

        res
    }

    pub fn get_network(&self, network_id: &str) -> &TestNetwork {
        self.networks
            .iter()
            .find(|network| network.model.id == network_id)
            .expect("Failed to find network")
    }

    pub async fn check_entity(&self, data: PushData, network_id: String) {
        let db: &DatabaseConnection = &self.db_connection;

        match data {
            PushData::Address(address) => {
                let result =
                    address::Entity::find_by_id((network_id.clone(), address.address.clone()))
                        .all(db)
                        .await
                        .expect("Failed to find address by id");

                assert_eq!(result.len(), 1);

                let address_model = result.first().unwrap();
                assert_eq!(address_model.address, address.address);
                assert_eq!(address_model.network_id, network_id);
                assert_eq!(address_model.case_id, address.case_id);
                assert_eq!(address_model.reporter_id, address.reporter_id);
                assert_eq!(address_model.risk, address.risk as i16);
                assert_eq!(address_model.category, address.category.into());
                assert_eq!(
                    address_model.confirmations,
                    address.confirmations.to_string()
                );
            }
            PushData::Asset(asset) => {
                let result = asset::Entity::find_by_id((
                    network_id,
                    asset.address.clone(),
                    asset.asset_id.to_string(),
                ))
                .all(db)
                .await
                .expect("Failed to find asset by id");

                assert_eq!(result.len(), 1);

                let asset_model = result.first().unwrap();
                assert_eq!(asset_model.address, asset.address);
                assert_eq!(asset_model.id, asset.asset_id.to_string());
                assert_eq!(asset_model.case_id, asset.case_id);
                assert_eq!(asset_model.reporter_id, asset.reporter_id);
                assert_eq!(asset_model.risk, asset.risk as i16);
                assert_eq!(asset_model.category, asset.category.into());
                assert_eq!(asset_model.confirmations, asset.confirmations.to_string());
            }
            PushData::Case(case) => {
                let result = case::Entity::find_by_id((network_id, case.id.clone()))
                    .all(db)
                    .await
                    .expect("Failed to find case by id");

                assert_eq!(result.len(), 1);

                let case_model = result.first().unwrap();
                assert_eq!(case_model.id, case.id);
                assert_eq!(case_model.name, case.name);
                assert_eq!(case_model.url, case.url);
                assert_eq!(case_model.status, case.status.into());
                assert_eq!(case_model.reporter_id, case.reporter_id);
            }
            PushData::Reporter(reporter) => {
                let result = reporter::Entity::find_by_id((network_id, reporter.id.clone()))
                    .all(db)
                    .await
                    .expect("Failed to find reporter by id");

                assert_eq!(result.len(), 1);

                let reporter_model = result.first().unwrap();
                assert_eq!(reporter_model.account, reporter.account);
                assert_eq!(reporter_model.role, reporter.role.into());
                assert_eq!(reporter_model.status, reporter.status.into());
                assert_eq!(reporter_model.name, reporter.name);
                assert_eq!(reporter_model.url, reporter.url);
                assert_eq!(reporter_model.stake, reporter.stake.to_string());
                assert_eq!(
                    reporter_model.unlock_timestamp,
                    reporter.unlock_timestamp.to_string()
                );
            }
        }
    }

    pub async fn global_setup<T>(
        &self,
        sender: &RequestSender,
        event: EventName,
    ) -> Vec<TestData<T>>
    where
        TestData<T>: FromTestPayload,
    {
        let mut res = vec![];

        for network in &self.networks {
            let data = get_test_data(&network.network, network.model.chain_id.clone());

            self.send_events(sender, &data).await;

            let mut entities: Vec<TestData<T>> = data
                .iter()
                .filter(|p| event == p.event.name)
                .map(|p| TestData::<T>::from_payload(p, &network.model.id))
                .collect();

            res.append(&mut entities);
        }

        res
    }

    pub async fn send_events(&self, sender: &RequestSender, test_data: &Vec<PushPayload>) {
        let token = create_jwt("my_ultra_secure_secret");

        for payload in test_data {
            sender
                .send("events", &payload, &token)
                .await
                .expect("Failed to send event");

            sleep(Duration::from_millis(WAITING_INTERVAL)).await;
        }
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        self.stop_signal.notify_one();
    }
}

pub fn generate_configuration() -> Configuration {
    let mut configuration = Configuration::default();
    configuration.listener = "127.0.0.1:0".parse().expect("Failed to parse address");

    if env::var(METRICS_ENV_VAR).unwrap_or_default().eq("1") {
        configuration.enable_metrics = true;
    }

    // TODO: implement db docker setup script
    configuration.database_url = env::var("DATABASE_URL")
        .unwrap_or("postgresql://postgres:postgres@localhost:5432/explorer".to_string());

    configuration
}
