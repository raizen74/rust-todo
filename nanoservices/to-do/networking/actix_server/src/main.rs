mod api;
use actix_web::{App, HttpServer};
use to_do_dal::migrations::run_migrations;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    run_migrations().await;
    HttpServer::new(|| App::new().configure(api::views_factory))
        .workers(4)
        .bind("127.0.0.1:8001")?
        .run()
        .await
}
