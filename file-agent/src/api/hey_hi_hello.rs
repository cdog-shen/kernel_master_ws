use actix_web::HttpResponse;

pub async fn hey() -> HttpResponse {
    HttpResponse::Ok().body("hi hello! -from FileAgent".to_string())
}
