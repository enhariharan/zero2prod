use actix_web::{HttpRequest, Responder};
use uuid::Uuid;

pub async fn greet(req: HttpRequest) -> impl Responder {
    let request_id = Uuid::new_v4();
    log::info!("req {} - Greeting request", request_id);
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello, {}!", &name)
}
