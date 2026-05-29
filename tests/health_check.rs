use std::net::TcpListener;

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

fn spawn_app() -> String {
    // Giving the address as "127.0.0.1:0" will spawn the server on a random port in the local machine
    let tcp_listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to port");
    let port = tcp_listener.local_addr().unwrap().port();
    // Spawn the app in a separate thread
    let server = zero2prod::run(tcp_listener).expect("Failed to start server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}