use sqlx::postgres::PgConnection;
use sqlx::{Connection, Executor, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configuration::{DatabaseSettings, get_configuration};
use zero2prod::startup::run;


pub struct TestApp {
    pub address: String,
    pub connection_pool: sqlx::PgPool,
}

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    let url = format!("{}/health_check", &test_app.address);
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to send request");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    let url = format!("{}/subscriptions", &test_app.address);

    let client = reqwest::Client::new();

    let body = "name=Hariharan%20Narayanan&email=enhariharan%40gmail.com";
    let response = client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request");

    println!("Response status: {}", response.status());
    assert!(response.status().is_success());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.connection_pool)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.email, "enhariharan@gmail.com");
    assert_eq!(saved.name, "Hariharan Narayanan")
}

#[tokio::test]
async fn subscribe_returns_400_for_invalid_form_data() {
    let test_app = spawn_app().await;
    let url = format!("{}/subscriptions", &test_app.address);
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=Hariharan%20Narayanan", "missing the email"),
        ("email=enhariharan%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, expected_error) in test_cases {
        let response = client
            .post(url.clone())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            response.status(),
            400,
            "The API did not fail with the expected HTTP error code 400 Bad Request when the payload was {}",
            expected_error
        );
    }
}

async fn spawn_app() -> TestApp {
    // Giving the address as "127.0.0.1:0" will spawn the server on a random port in the local machine
    let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let port = tcp_listener.local_addr().unwrap().port();
    println!("Server is running on port {}", port);

    let mut configuration = get_configuration().expect("Failed to load configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    println!(
        "Using test database name {}",
        configuration.database.database_name
    );

    let connection_pool = configure_test_database(&configuration.database).await;
    sqlx::PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database");
    println!("Test database connection pool is ready");

    // Spawn the app in a separate thread
    let server = run(tcp_listener, connection_pool.clone()).expect("Failed to start server");
    let spawned_server = tokio::spawn(server);
    println!("App server is running");
    drop(spawned_server);

    let address = format!("http://127.0.0.1:{}", port);
    println!("Server is running at {}", address);

    TestApp {
        address,
        connection_pool,
    }
}

async fn configure_test_database(config: &DatabaseSettings) -> PgPool {
    let maintenance_db_settings = DatabaseSettings {
        username: "postgres".to_string(),
        password: "password".to_string(),
        database_name: "postgres".to_string(),
        ..config.clone()
    };
    let mut maintenance_db_connection =
        PgConnection::connect(&maintenance_db_settings.connection_string())
            .await
            .expect("Failed to connect to maintenance database");
    println!("Connected to maintenance database");

    maintenance_db_connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create test database");
    println!("Test database created: {}", config.database_name);

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to test database");
    println!("Connected to test database");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run migrations on test database");
    println!("Migrations applied to test database");

    connection_pool
}
