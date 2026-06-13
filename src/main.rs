use std::io::Stdout;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::Application;
use zero2prod::telemetry::init_tracing_subscriber;

const TRACING_COMPONENT_NAME: &str = "zero2prod";
const TRACING_SPAN_WRITER: fn() -> Stdout = || std::io::stdout();
const TRACING_ENV_FILTER: &str = "info";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing_subscriber(
        TRACING_COMPONENT_NAME.into(),
        TRACING_SPAN_WRITER,
        TRACING_ENV_FILTER.into(),
    );

    // Set up server configuration and database connection
    let configuration = get_configuration().expect("Failed to load configuration");
    let application: Application = Application::build(configuration).await?;
    application.run_until_stopped().await?;

    Ok(())
}
