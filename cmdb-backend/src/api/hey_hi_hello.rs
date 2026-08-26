use actix_web::HttpResponse;

pub async fn hey() -> HttpResponse {
    HttpResponse::Ok().body("hi hello! -from CMDB".to_string())
}
