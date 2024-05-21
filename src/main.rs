use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use agent::youtube_bot::summarize_video;
mod agent;

async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("templates/index.html"))
}

async fn summarizer(id: String) -> impl Responder {
    let summary = summarize_video(&id).await;
    HttpResponse::Ok().body(summary)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 4004;
    println!("Starting web server at port {}", port);
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/", web::post().to(summarizer))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
