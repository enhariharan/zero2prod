use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });

    // Try to login with a wrong password
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/login");

    // Follow the redirect to the login page
    let html_page = app.get_login_html().await;
    assert!(html_page.contains("<div><p><i>Authentication failed</i></p></div>"));

    // Reload the login page - flash message should be gone
    let html_page = app.get_login_html().await;
    assert!(!html_page.contains("<div><p><i>Authentication failed</i></p></div>"));
}
