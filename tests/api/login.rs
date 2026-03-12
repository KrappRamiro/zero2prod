use crate::helpers::{assert_response_is_redirect_to, spawn_app};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;

    // Act - Part 1 - Try to login
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });
    let response = app.post_login(&login_body).await;

    // Assert - Part 1 - Check the redirection
    assert_response_is_redirect_to(&response, "/login");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_login_html().await;

    // Assert - Part 2 - Check that the page contains an error message
    assert!(html_page.contains(r#"<p><i>authentication failed</i></p>"#));

    // Act - Part 3 - Reload the login page
    let html_page = app.get_login_html().await;

    // Assert - Part 3 - Check that the page no longer has the flash message
    assert!(!html_page.contains(r#"<p><i>authentication failed</i></p>"#));
}
