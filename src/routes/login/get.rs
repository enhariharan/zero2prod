use actix_web::cookie::{Cookie, time::Duration};
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse};

pub async fn login_form(request: HttpRequest) -> HttpResponse {
    let error_html: String = match request.cookie("_flash") {
        None => "".into(),
        Some(cookie) => format!(
            "<p><i>{}</i></p>",
            htmlescape::encode_minimal(cookie.value())
        ),
    };

    let mut removal_cookie = Cookie::new("_flash", "");
    removal_cookie.set_max_age(Duration::ZERO);

    let mut response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .cookie(Cookie::build("_flash", "").max_age(Duration::ZERO).finish())
        .body(format!(include_str!("login.html"), error_html = error_html));
    response
        .add_removal_cookie(&Cookie::new("_flash", ""))
        .unwrap();
    response
}
