use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
mod api_key;
mod hashing;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/api_key/generate")]
async fn generate_api_key() -> impl Responder {
    let api_key = api_key::generate_api_key();
    api_key::store_api_key(api_key.clone());
    HttpResponse::Ok().body(api_key)
}

#[post("/api_key/verify")]
async fn verify_api_key(req_body: String) -> impl Responder {
    let is_valid = api_key::verify_api_key(&req_body);
    HttpResponse::Ok().body(is_valid.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    api_key::load_api_keys();

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(generate_api_key)
            .service(verify_api_key)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
