//! src/configuration.rs

use secrecy::Secret;
use serde_aux::field_attributes::deserialize_number_from_string;

use secrecy::ExposeSecret;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    // Converts from str to u16 in case we up an environment variable
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn connect_options(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
            .database(&self.database_name)
    }
}

#[derive(serde::Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    // Converts from str to u16 in case we up an environment variable
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

/// The possible runtime environment for our application
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    // We cant use the Enum directly, so this helps us get the Enum as a str
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

/// This helps us parse the value of APP_ENVIRONMENT safely.
/// When we read APP_ENVIRONMENT from the OS using std::env::var, we get a raw String.
/// Problem of that, we rely on the Environment enum, so the value is not safe, it could be
/// something like Banana ! And that would be a disaster.
/// Here, TryFrom allows us to convert that unsafe string (which could be "local", "production" or
/// "banana" into our safe Enum)
///
/// This also allows use to call try_into()
impl TryFrom<String> for Environment {
    type Error = String; // <--- The trait *demands* you define this
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. \
Use either `local` or `production`.",
                other
            )),
        }
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");
    // Detect the running environment, defaults to `local` if unspecified
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        // Intentar conseguir la var de entorno, si no se
        // consigue, bueno, usamos "local" como reemplazo
        .unwrap_or_else(|_| "local".into())
        // Ahora, fijate si la variable APP_ENVIRONMENT tiene algun valor que nos
        // sirva, o si nos pusieron mierda.
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");
    let environment_filename = format!("{}.yaml", environment.as_str());

    // Init the config reader
    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        // Add in settings from environment variables (with a prefix of APP and
        // '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;
    // Try to convert the configuration values it read into our Settings type
    settings.try_deserialize::<Settings>()
}
