use secrecy::SecretString;
use sqlx::postgres::{PgConnection, PgPoolOptions};
use sqlx::{Connection, Executor, PgPool};
use std::io::Stdout;
use std::net::TcpListener;
use std::sync::LazyLock;
use uuid::Uuid;
use zero2prod::configuration::{DatabaseSettings, get_configuration};
use zero2prod::domain::SubscriberEmail;
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;
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
}

pub async fn spawn_app() -> TestApp {
    //init tracing
    LazyLock::force(&TRACING);

    // Giving the address as "127.0.0.1:0" will spawn the server on a random port in the local machine
    let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let port = tcp_listener.local_addr().unwrap().port();
    tracing::info!("Server is running on port {}", port);

    let mut configuration = get_configuration().expect("Failed to load configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    tracing::info!(
        "Using test database name {}",
        configuration.database.database_name
    );

    let connection_pool = configure_test_database(&configuration.database).await;
    PgConnection::connect_with(&configuration.database.connection_options())
        .await
        .expect("Failed to connect to database");

    tracing::info!("Test database connection pool is ready");

    let timeout = std::time::Duration::from_millis(configuration.email_client.timeout_milliseconds);
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        SubscriberEmail::parse(configuration.email_client.sender_email)
            .expect("Could not parse sender email"),
        configuration.email_client.authorization_token.clone(),
        timeout,
    );

    // Spawn the app in a separate thread
    let server =
        run(tcp_listener, connection_pool.clone(), email_client).expect("Failed to start server");
    let spawned_server = tokio::spawn(server);
    tracing::info!("App server is running");
    drop(spawned_server);

    let address = format!("http://127.0.0.1:{}", port);
    tracing::info!("Server is running at {}", address);

    TestApp {
        address,
        connection_pool,
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
    tracing::debug!("Connected to maintenance database");

    maintenance_db_connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create test database");
    tracing::debug!("Test database created: {}", config.database_name);

    let connection_pool = PgPoolOptions::new()
        .connect_with(config.connection_options())
        .await
        .expect("Failed to connect to test database");
    tracing::debug!("Connected to test database");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run migrations on test database");
    tracing::debug!("Migrations applied to test database");

    connection_pool
}
