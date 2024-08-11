use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub app: AppSettings,
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
}

#[derive(Deserialize, Clone)]
pub struct AppSettings {
    pub host: String,
    pub port: u16,
    pub client_url: String,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,
    pub database_name: String,
    pub require_ssl: bool,
}

#[derive(Deserialize, Clone)]
pub struct JwtSettings {
    pub issuer: String,
    pub audience: String,
    pub access_token_secret: Secret<String>,
    pub refresh_token_secret: Secret<String>,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .username(&self.username)
            .password(self.password.expose_secret())
            .host(&self.host)
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.database_name)
    }
}

enum Environment {
    Local,
    Production 
}

impl Environment {
    pub fn as_str(&self) -> &str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    } 
}

impl TryFrom<String> for Environment {
    type Error = String;
 
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local"  => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!("{} is not a valid environment.", other))
        }
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let base_dir = std::env::current_dir().expect("Failed to parse current directory.");
    let config_dir = base_dir.join("configurations");
    
    let env: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or("local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    let env_file = format!("{}.yaml", env.as_str());

    let config = config::Config::builder()
        .add_source(
            config::File::from(
                config_dir.join("base.yaml")
            )
        )
        .add_source(
            config::File::from(
                config_dir.join(env_file)
            )
        )
        .add_source(
            config::Environment::with_prefix("APP")
            .prefix("_")
            .separator("__")
        )
        .build()?;

    config.try_deserialize()
}