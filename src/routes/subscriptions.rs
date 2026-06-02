use actix_web::web::{Data, Form};
use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

fn is_valid_name(s: &String) -> bool {
    let is_empty_or_whitespace = s.trim().is_empty();

    let is_too_long = s.graphemes(true).count() > 256;

    let forbidden_characters = ['/', '(', ')', '{', '}', '[', ']', '\\', '<', '>', '"'];
    let contains_forbidden_characters = s.chars().any(|c| forbidden_characters.contains(&c));

    !(is_empty_or_whitespace || is_too_long || contains_forbidden_characters)
}

#[tracing::instrument(
    name = "Add a new subscriber",
    skip(form, connection_pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    connection_pool: web::Data<PgPool>,
) -> HttpResponse {
    tracing::info_span!("Saving new subscriber details into DB");

    if is_valid_name(&form.name) {
        return HttpResponse::BadRequest().finish();
    }

    match insert_new_subscriber(&connection_pool, &form).await {
        Ok(_) => {
            tracing::info!("New subscriber saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("Error saving new subscriber: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(name = "Insert new subscriber into DB", skip(connection_pool, form))]
async fn insert_new_subscriber(
    connection_pool: &Data<PgPool>,
    form: &Form<FormData>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, created_at) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection_pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Faile dto execute query: {:?}", e);
        e
    })?;

    Ok(())
}
