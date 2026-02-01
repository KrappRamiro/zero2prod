//! src/configuration.rs

use secrecy::Secret;

use secrecy::ExposeSecret;
#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
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
        .build()?;
    // Try to convert the configuration values it read into our Settings type
    settings.try_deserialize::<Settings>()
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }
}
