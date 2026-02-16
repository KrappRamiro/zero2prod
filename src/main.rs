use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::{
    configuration::{self, get_configuration},
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber_as_global_default},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber_as_global_default(subscriber);
    // Panic if we cant read configuration
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool =
        PgPoolOptions::new().connect_lazy_with(configuration.database.connect_options());
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool, email_client)?.await
}
