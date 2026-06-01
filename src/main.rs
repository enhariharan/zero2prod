use std::io::Stdout;
use std::net::TcpListener;
use sqlx::postgres::PgPoolOptions;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    const TRACING_COMPONENT_NAME: &str = "zero2prod";
    const TRACING_SPAN_WRITER: fn() -> Stdout = || std::io::stdout();
    const TRACING_ENV_FILTER: &str = "info";

    telemetry::init_tracing_subscriber(
        TRACING_COMPONENT_NAME.into(),
        TRACING_SPAN_WRITER,
        TRACING_ENV_FILTER.into(),
    );

    // Set up server configuration and database connection
    let configuration = get_configuration().expect("Failed to load configuration");
    let connection_pool = PgPoolOptions::new()
        .connect_lazy_with(configuration.database.connection_options());

    // Start the app server
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let tcp_listener = TcpListener::bind(address).expect("Failed to bind to port");
    let url = format!(
        "http://{}:{}",
        configuration.application.host, configuration.application.port
    );
    tracing::info!("Server running at {}", url);
    run(tcp_listener, connection_pool)?.await
}
