use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
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
    pub port: u16,
    pub(crate) test_user: TestUser,
    pub api_client: reqwest::Client,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(format!("{}/subscriptions", &self.address).as_str())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to post subscriptions")
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/newsletters", &self.address))
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/login", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_login_html(&self) -> String {
        self.api_client
            .get(&format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
            .text()
            .await
            .unwrap()
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, plain_text }
    }

    pub async fn get_admin_dashboard_html(&self) -> String {
        self.api_client
            .get(&format!("{}/admin/dashboard", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
            .text()
            .await
            .unwrap()
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
    let application_port = application.port();
    let address = format!(
        "http://{}:{}",
        configuration.application.host, application_port
    );
    tracing::info!("***** Server will start on {}", address);
    let spawned_server = tokio::spawn(application.run_until_stopped());
    drop(spawned_server);

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();
    let test_app = TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        connection_pool: get_connection_pool(&configuration.database),
        email_server,
        test_user: TestUser::generate(),
        api_client: client,
    };
    test_app.test_user.store(&test_app.connection_pool).await;
    test_app
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

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        )
        .hash_password(self.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

        sqlx::query!(
            "INSERT INTO users (user_id, username, password_hash) VALUES ($1, $2, $3)",
            self.user_id,
            self.username,
            password_hash,
        )
        .execute(pool)
        .await
        .expect("Failed to create test users.");
    }
}

pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}
