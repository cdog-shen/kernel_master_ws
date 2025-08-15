use actix_web::HttpResponse;

pub async fn generate() -> HttpResponse {
    HttpResponse::ImATeapot().body("I am a teapot so i DID NOT finish this API yet :)".to_string())
}
