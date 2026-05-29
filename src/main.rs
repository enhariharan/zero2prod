use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to load configuration");
    let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let url = format!("http://127.0.0.1:{}", configuration.application_port);
    println!("Server running at {}", url);
    run(tcp_listener)?.await
}
