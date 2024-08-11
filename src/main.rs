use test_rs::{configurations::get_config, startup::Application, telemetry::{get_subscriber, init_subscriber}};

#[tokio::main]
async fn main() -> std::io::Result<()>{
    let config = get_config()
        .expect("failed to parse configurations.");

    let sub = get_subscriber("server", "info", std::io::stdout);
    init_subscriber(sub);

    let app = Application::build(config).await?;

    app.run_until_stopped().await
}
