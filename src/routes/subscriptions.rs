use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, connection: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    log::info!(
        "req {} - Received subscription request: {:?}",
        request_id,
        form
    );
    match sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, created_at) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .await
    {
        Ok(_) => {
            log::info!("req {} -Subscription saved", request_id);
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            log::error!("req {} - Error saving subscription: {}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
