use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    let body = "name=Hariharan%20Narayanan&email=enhariharan%40gmail.com";

    let response = test_app.post_subscriptions(body.to_string()).await;

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
async fn subscribe_returns_400_when_fields_are_present_but_invalid() {
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=enhariharan%40gmail.com", "empty name"),
        ("name=enhariharan&email=", "empty email"),
        ("name=enhariharan&email=invalid_email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = test_app.post_subscriptions(body.to_string()).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with the expected HTTP error code 400 Bad Request when the payload was {}",
            description
        );
    }
}

#[tokio::test]
async fn subscribe_returns_400_for_invalid_form_data() {
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=Hariharan%20Narayanan", "missing the email"),
        ("email=enhariharan%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, expected_error) in test_cases {
        let response = test_app.post_subscriptions(body.to_string()).await;

        assert_eq!(
            response.status(),
            400,
            "The API did not fail with the expected HTTP error code 400 Bad Request when the payload was {}",
            expected_error
        );
    }
}
