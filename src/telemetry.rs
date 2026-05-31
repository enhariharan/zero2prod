use std::io::Stdout;
use tracing::Subscriber;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn init_tracing_subscriber(
    tracing_component_name: String,
    tracing_span_writer: fn() -> Stdout,
    tracing_env_filter: String,
) {
    // Setup all logging to be routed through LogTracer
    LogTracer::init().expect("Failed to set logger");

    let tracing_subscriber = get_tracing_subscriber(
        tracing_component_name,
        tracing_span_writer,
        tracing_env_filter,
    );

    set_global_default(tracing_subscriber)
        .expect("Failed to set global default tracing subscriber");
}

fn get_tracing_subscriber(
    tracing_component_name: String,
    tracing_span_writer: fn() -> Stdout,
    tracing_env_filter: String,
) -> impl Subscriber + Send + Sync {
    // Setup tracing.
    // Use the tracing level set in the RUST_LOG env variable. Else, set tracing level to info
    let tracing_env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(tracing_env_filter.as_str()));
    let tracing_formatting_layer =
        BunyanFormattingLayer::new(tracing_component_name.into(), tracing_span_writer);
    Registry::default()
        .with(tracing_env_filter)
        .with(JsonStorageLayer)
        .with(tracing_formatting_layer)
}
