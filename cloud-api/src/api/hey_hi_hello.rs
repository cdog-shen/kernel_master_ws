use actix_web::HttpResponse;

pub async fn hey() -> HttpResponse {
    HttpResponse::Ok().body("hi hello! -from cloud_api".to_string())
}