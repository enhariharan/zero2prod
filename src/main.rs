use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

use env_logger::Env;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let configuration = get_configuration().expect("Failed to load configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database");
    let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let url = format!("http://127.0.0.1:{}", configuration.application_port);
    println!("Server running at {}", url);
    run(tcp_listener, connection_pool)?.await
}
