use actix_web::{get, web, App, HttpServer, Responder};
use agent::youtube_agent::summarize_video;
mod agent;

#[get("/{video_id}")]
async fn summarize(video_id: web::Path<String>) -> impl Responder {
    let res = summarize_video(&video_id).await;
    res
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Starting web server at port 4004");
    HttpServer::new(|| App::new().service(summarize))
        .bind(("0.0.0.0", 4004))
        .unwrap()
        .run()
        .await
}
