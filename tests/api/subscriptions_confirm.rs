use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
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
    // Get the first and only email the email mock server recieved
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);
    let response = reqwest::get(confirmation_links.html).await.unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    // Act
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Assert
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_twice_just_returns_a_200() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    // Act
    reqwest::get(confirmation_links.html.as_ref())
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    reqwest::get(confirmation_links.html.as_ref())
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Assert
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
}

#[tokio::test]
async fn using_a_well_formatted_but_non_existing_token_fails_with_a_401() {
    // Arrange
    let app = spawn_app().await;

    // Act
    // We completely bypass the subscription phase and just make a direct GET request
    // to the confirmation endpoint with a completely made-up token.
    let response = reqwest::get(&format!(
        "{}/subscriptions/confirm?subscription_token=nonexisting25charstoken00",
        app.address
    ))
    .await
    .unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn using_an_incorrectly_formatted_token_fails_with_a_400() {
    // Arrange
    let app = spawn_app().await;

    // Act
    // Yes, i smashed my keyboard
    let response = reqwest::get(&format!(
    "{}/subscriptions/confirm?subscription_token=ALKJSHSHLJKAH:JKLDlhjk_<F23>_A+SDOI_@!()IH_CANY_8912uyf_app89gu;3pf9ewtp8usifqpwnh6rt_972uend-typ8",
        app.address
    ))
    .await
    .unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}
