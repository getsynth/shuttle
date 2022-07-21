use actix_web::{get, web, App, HttpServer, Responder};

#[get("/")]
async fn hello_world(name: web::Path<String>) -> impl Responder {
    format!("Hello, world!")
}

#[shuttle_service::main]
async fn actix() -> shuttle_service::ShuttleActix {
    let actix = HttpServer::new(move || App::new().service(hello_world));

    Ok(actix)
}
