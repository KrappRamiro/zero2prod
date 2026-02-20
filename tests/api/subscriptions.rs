use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use crate::helpers::{TestApp, spawn_app};

#[tokio::test]
async fn suscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;

    // FIXME: This should be generated better, not like a string like this
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let response = app.post_subscriptions(body.into()).await;
    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_allows_a_user_subscribing_two_times() {
    // Arrange
    let app = spawn_app().await;

    // FIXME: This should be generated better, not like a string like this
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.email_server)
        .await;

    // Act
    let response_1 = app.post_subscriptions(body.into()).await;
    let response_2 = app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(200, response_1.status().as_u16());
    assert_eq!(200, response_2.status().as_u16());
}

#[tokio::test]
async fn a_user_that_subscribes_two_times_has_the_same_token() {
    // Arrange
    let app = spawn_app().await;

    // FIXME: This should be generated better, not like a string like this
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;
    app.post_subscriptions(body.into()).await;

    let email_requests = app.email_server.received_requests().await.unwrap();
    let first_request_body = &email_requests[0].body;
    let second_request_body = &email_requests[1].body;

    // Assert
    // Lets check that they are the same token
    assert_eq!(first_request_body, second_request_body);
}

#[tokio::test]
async fn a_user_that_subscribes_two_times_is_not_duplicated_in_the_database() {
    // Arrange
    let app = spawn_app().await;

    // FIXME: This should be generated better, not like a string like this
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;
    app.post_subscriptions(body.into()).await;

    // Verify we didn't insert a duplicate row
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_all(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");

    assert_eq!(
        saved.len(),
        1,
        "Expected exactly 1 subscription record in the database."
    );
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let app = spawn_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    // arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = app.post_subscriptions(body.into()).await;

        //Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}

#[tokio::test]
async fn suscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.post_subscriptions(invalid_body.into()).await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The api did not fail with 400 Bad Request when the payload was {}",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
    // Mocks asserts on drop, no need to do anything
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        // We are not setting any expectations because the test
        // is focused on looking if there are links in the email
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
    // Get the first intercepted request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let links = app.get_confirmation_links(email_request);

    // The two links should be identical.
    // Dont worry, we already check that they exist, thanks to the assert_eq in get_confirmation_links
    assert_eq!(links.html, links.plain_text)
}
