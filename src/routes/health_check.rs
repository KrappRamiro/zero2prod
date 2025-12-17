use actix_web::HttpResponse;

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
