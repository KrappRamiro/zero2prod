use crate::configuration::{DatabaseSettings, Settings};
use crate::{
    email_client::EmailClient,
    routes::{health_check, subscribe},
};
use actix_web::{App, HttpServer, dev::Server, web};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configuration.connect_options())
}

/// `Application` works as a wrapper for actix_web `dev::Server`.
/// It was made because `dev::Server` does not tell us in which port the app was allocated,
/// so if we wrap it in an struct with the port alongside it, we long have that issue.
/// Why do we need to know the port? The tests need them.
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    /// Given a configuration of type `Settings`:
    /// 1. A database connection pool is started (could be lazy, check `get_connection_pool`
    ///    implementation)
    /// 2. An email client is configured
    /// 3. A server is started with `run`, which can be accesed using `run_until_stopped`
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");

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
        let port = listener.local_addr().unwrap().port();

        let server = run(listener, connection_pool, email_client)?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// This function only returns when the application is stopped
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // web::Data wraps our connection in an Arc<T>
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
