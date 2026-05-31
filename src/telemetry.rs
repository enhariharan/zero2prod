use std::io::Stdout;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::{EnvFilter, Registry};

pub fn init_tracing_subscriber() {
    // Setup all logging to be routed through LogTracer
    LogTracer::init().expect("Failed to set logger");

    let tracing_subscriber = get_tracing_subscriber();
    set_global_default(tracing_subscriber)
        .expect("Failed to set global default tracing subscriber");
}

fn get_tracing_subscriber() -> Layered<
    BunyanFormattingLayer<fn() -> Stdout>,
    Layered<JsonStorageLayer, Layered<EnvFilter, Registry>>,
> {
    // Setup tracing.
    // Use the tracing level set in the RUST_LOG env variable. Else, set tracing level to info
    let tracing_env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    const TRACING_COMPONENT_NAME: &str = "zero2prod";
    const TRACING_SPAN_WRITER: fn() -> Stdout = || std::io::stdout();
    let tracing_formatting_layer =
        BunyanFormattingLayer::new(TRACING_COMPONENT_NAME.into(), TRACING_SPAN_WRITER);
    Registry::default()
        .with(tracing_env_filter)
        .with(JsonStorageLayer)
        .with(tracing_formatting_layer)
}
