mod api;
use actix_web::{App, HttpServer};
use actix_cors::Cors;
use to_do_dal::migrations::run_migrations;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    run_migrations().await;
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .configure(api::views_factory)
            .wrap(cors)
    })
    .workers(4)
    .bind("127.0.0.1:8001")?
    .run()
    .await
}
