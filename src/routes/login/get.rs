use crate::startup::HmacSecret;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, web};
use hmac::{Hmac, KeyInit, Mac};
use secrecy::ExposeSecret;

pub async fn login_form(
    query: Option<web::Query<QueryParams>>,
    secret: web::Data<HmacSecret>,
) -> HttpResponse {
    match query {
        Some(query) => match query.0.verify(&secret) {
            Ok(error) => format!(
                "<div class='alert alert-danger'>{}</div>",
                htmlescape::encode_minimal(&error)
            ),
            Err(e) => {
                tracing::warn!(
                    error.message = %e,
                    error.cause_chain = ?e,
                    "Failed to verify query parameters using the HMAC tag."
                );
                "".into()
            }
        },
        None => "".into(),
    };
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("login.html"))
}

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(&self.tag)?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));

        let mut mac =
            Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;

        Ok(self.error)
    }
}
