use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

pub fn get_subscriber<S>(
    app_name : &str,
    log_level : &str,
    sink : S
) -> impl Subscriber + Send + Sync
    where S : for<'a> MakeWriter<'a> + Send + Sync + 'static
    {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new(log_level));

    let formatting_layer = BunyanFormattingLayer::new(
        app_name.into(),
        sink
    );

    Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .with(JsonStorageLayer)
}

pub fn init_subscriber(sub : impl Subscriber + Send + Sync) {
    LogTracer::init()
        .expect("failed to initialize logger.");

    tracing::subscriber::set_global_default(sub)
        .expect("failed to initialize logger.");
}