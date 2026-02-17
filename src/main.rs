use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::{
    configuration::get_configuration,
    email_client::EmailClient,
    startup::{Application, run},
    telemetry::{get_subscriber, init_subscriber_as_global_default},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber_as_global_default(subscriber);
    // Panic if we cant read configuration
    let configuration = get_configuration().expect("Failed to read configuration");

    let application = Application::build(configuration).await?;

    application.run_until_stopped().await?;
    Ok(())
}
