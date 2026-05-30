use crate::routes::*;
use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use std::net::TcpListener;

pub fn run(tcp_listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/subscriptions", web::post().to(subscribe))
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(greet))
            .app_data(connection.clone())
    })
    .listen(tcp_listener)?
    .run();

    Ok(server)
}
