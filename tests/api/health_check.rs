use crate::helpers::spawn_app;

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
