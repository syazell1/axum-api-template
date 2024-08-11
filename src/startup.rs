use std::sync::Arc;

use axum::http::HeaderValue;
use axum::{routing::get, serve::Serve, Router};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Method;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::utils::password_hasher::ServerPwdHasher;
use crate::{
    app_state::AppState,
    configurations::{DatabaseSettings, JwtSettings, Settings},
    db::DbPool,
    features::{auth::controller::auth_routes, health_check::controller::health_check},
};

pub struct Application {
    server: Serve<Router, Router>,
    port: u16,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        let address = TcpListener::bind(format!("{}:{}", config.app.host, config.app.port))
            .await
            .expect("Failed to bind address");

        let port = address.local_addr().unwrap().port();
        let pool = get_db_pool(&config.database);
        let pwd_hasher = ServerPwdHasher;
        let app_routes = get_app_routes(config.app.client_url, pool, config.jwt, pwd_hasher);
        let server = axum::serve(address, app_routes);

        Ok(Self { server, port })
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

fn get_app_routes(
    client_url: String,
    pool: PgPool,
    jwt_settings: JwtSettings,
    pwd_hasher: ServerPwdHasher,
) -> Router {
    let app_state = Arc::new(AppState {
        pool: DbPool { pool },
        jwt_settings,
        pwd_hasher,
    });

    Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/health_check", get(health_check))
                .nest("/auth", auth_routes()),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(client_url.parse::<HeaderValue>().unwrap())
                .allow_credentials(true)
                .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ]),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

pub fn get_db_pool(config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(config.with_db())
}
