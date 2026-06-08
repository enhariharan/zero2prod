use secrecy::SecretString;
use sqlx::postgres::{PgConnection, PgPoolOptions};
use sqlx::{Connection, Executor, PgPool};
use std::io::Stdout;
use std::sync::LazyLock;
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::configuration::{DatabaseSettings, get_configuration};
use zero2prod::domain::SubscriberEmail;
use zero2prod::email_client::EmailClient;
use zero2prod::startup::{Application, get_connection_pool};
use zero2prod::telemetry;

const TRACING_COMPONENT_NAME: &str = "zero2prod_test";
const TRACING_SPAN_WRITER: fn() -> Stdout = || std::io::stdout();
const TRACING_ENV_FILTER: &str = "debug";

static TRACING: LazyLock<()> = LazyLock::new(|| {
    telemetry::init_tracing_subscriber(
        TRACING_COMPONENT_NAME.into(),
        TRACING_SPAN_WRITER,
        TRACING_ENV_FILTER.into(),
    );
});

pub struct TestApp {
    pub address: String,
    pub connection_pool: sqlx::PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{}/subscriptions", &self.address).as_str())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to post subscriptions")
    }
}

pub async fn spawn_app() -> TestApp {
    //init tracing
    LazyLock::force(&TRACING);

    let email_server = MockServer::start().await;
    let configuration = {
        let mut c = get_configuration().expect("Failed to load configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri().to_string();
        c
    };

    configure_test_database(&configuration.database).await;
    PgConnection::connect_with(&configuration.database.connection_options())
        .await
        .expect("Failed to connect to database");

    let timeout = std::time::Duration::from_millis(configuration.email_client.timeout_milliseconds);
    EmailClient::new(
        configuration.email_client.base_url.clone(),
        SubscriberEmail::parse(configuration.email_client.sender_email.clone())
            .expect("Could not parse sender email"),
        configuration.email_client.authorization_token.clone(),
        timeout,
    );

    // Spawn the app in a separate thread
    tracing::info!(
        "Server will start on {}:{}",
        configuration.application.host,
        configuration.application.port
    );
    let application: Application = Application::build(configuration.clone())
        .await
        .expect("Failed to start app server");
    let address = format!(
        "http://{}:{}",
        configuration.application.host,
        application.port()
    );
    tracing::info!("***** Server will start on {}", address);
    let spawned_server = tokio::spawn(application.run_until_stopped());
    drop(spawned_server);

    TestApp {
        address,
        connection_pool: get_connection_pool(&configuration.database),
        email_server,
    }
}

async fn configure_test_database(config: &DatabaseSettings) -> PgPool {
    let maintenance_db_settings = DatabaseSettings {
        username: "postgres".to_string(),
        password: SecretString::new(Box::from("password".to_string())),
        database_name: "postgres".to_string(),
        ..config.clone()
    };
    let mut maintenance_db_connection =
        PgConnection::connect_with(&maintenance_db_settings.connection_options())
            .await
            .expect("Failed to connect to maintenance database");

    maintenance_db_connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create test database");

    let connection_pool = PgPoolOptions::new()
        .connect_with(config.connection_options())
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run migrations on test database");
    tracing::debug!(
        "Connected to test database - [{}] - and applied migrations",
        config.database_name
    );

    connection_pool
}
