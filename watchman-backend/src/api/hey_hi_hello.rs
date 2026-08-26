use actix_web::HttpResponse;

pub async fn hey() -> HttpResponse {
    HttpResponse::Ok().body("hi hello!".to_string())
}
