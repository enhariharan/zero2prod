use std::net::TcpListener;
use zero2prod::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let tcp_listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to port");
    let port = tcp_listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{}", port);
    println!("Server running at {}", url);
    run(tcp_listener)?.await
}
