use actix_web::{HttpRequest, HttpResponse};
use uuid::Uuid;

pub async fn health_check(_req: HttpRequest) -> HttpResponse {
    let request_id = Uuid::new_v4();
    log::info!("req {} - Health check", request_id);
    HttpResponse::Ok().finish()
}
