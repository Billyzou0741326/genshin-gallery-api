use actix_web::{get, HttpResponse, Responder};

#[get("/api/health")]
pub async fn api_health() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("{}")
}
