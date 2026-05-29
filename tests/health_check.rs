use sqlx::Connection;
use sqlx::postgres::PgConnection;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    let url = format!("{}/health_check", &address);
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
    let app_address = spawn_app();
    let url = &format!("{}/subscriptions", &app_address);
    let configuration = get_configuration().expect("Failed to load configuration");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to the database");

    let client = reqwest::Client::new();

    let body = "name=Hariharan%20Narayanan&email=enhariharan%40gmail.com";
    let response = client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.email, "enhariharan@gmail.com");
    assert_eq!(saved.name, "Hariharan Narayanan")
}

#[tokio::test]
async fn subscribe_returns_400_for_invalid_form_data() {
    let app_address = spawn_app();
    let url = &format!("{}/subscriptions", &app_address);
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=Hariharan%20Narayanan", "missing the email"),
        ("email=enhariharan%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, expected_error) in test_cases {
        let response = client
            .post(url)
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

fn spawn_app() -> String {
    // Giving the address as "127.0.0.1:0" will spawn the server on a random port in the local machine
    let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let port = tcp_listener.local_addr().unwrap().port();
    // Spawn the app in a separate thread
    let server = run(tcp_listener).expect("Failed to start server");
    let _spawned_server = tokio::spawn(server);
    drop(_spawned_server);
    format!("http://127.0.0.1:{}", port)
}
