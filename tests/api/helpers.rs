use std::sync::LazyLock;

use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, Executor, PgPool};
use test_rs::{configurations::{get_config, DatabaseSettings}, db::DbPool, features::auth::{domain::Credentials, repository::create_user}, startup::{get_db_pool, Application}, telemetry::{get_subscriber, init_subscriber}, utils::{password_hasher::ServerPwdHasher, randomizer::generate_random_string}};
use uuid::Uuid;

static TRACING : LazyLock<()> = LazyLock::new(|| {
    let app_name = "test";
    let log_level = "info";

    if std::env::var("TEST_LOG").is_ok() {
        let sub = get_subscriber(app_name, log_level, std::io::stdout);
        init_subscriber(sub)
    } else {
        let sub = get_subscriber(app_name, log_level, std::io::sink);
        init_subscriber(sub)
    }
});

pub struct TestApp {
    pub http_client : reqwest::Client,
    pub pool : PgPool,
    pub address : String,
    pub test_user : TestUser,
    pub port : u16 
}

impl TestApp {
    pub async fn login_user<T : serde::Serialize>(&self, body : T) -> reqwest::Response {
        self.http_client
            .post(format!("{}/auth/login", self.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to send login request.")
    }
}

pub async fn spawn_app () -> TestApp {
    LazyLock::force(&TRACING);

    let config = {
        let mut c = get_config()
            .expect("Failed to parse configuraiton.");
        c.app.port = 0;
        c.database.database_name = Uuid::new_v4().to_string();

        c
    };

    configure_db(&config.database).await;
    let pool = get_db_pool(&config.database);
    let http_client = reqwest::Client::new();
    let app = Application::build(config)
        .await
        .expect("Failed to build application.");
    let port = app.get_port();
    let _ = tokio::spawn(app.run_until_stopped());

    TestApp {
        http_client,
        pool,
        test_user : TestUser::generate(),
        address : format!("http://localhost:{}/api", port),
        port
    }

}

#[derive(Serialize)]
pub struct TestUser {
    pub username : String,
    pub password : String
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            username : generate_random_string(12),
            password : generate_random_string(12)
        }
    }

    pub async fn store_user(&self, pool : &PgPool) {
        let pwd_hasher = ServerPwdHasher;
        let db_ctx = DbPool { pool : pool.clone() };

        let credentials = Credentials { username : self.username.to_string(), password: self.password.to_string() };

        create_user(&credentials, &db_ctx, &pwd_hasher)
            .await
            .expect("Failed to create test user.");
    }
}

async fn configure_db(config : &DatabaseSettings) {
    let pool = PgPoolOptions::new().connect_with(config.without_db())
        .await
        .expect("Failed to connect to database.");

    pool
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    let pool = PgPoolOptions::new().connect_with(config.with_db())
        .await
        .expect("Failed to connect to database.");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations.")
}