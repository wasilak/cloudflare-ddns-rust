mod libs;
mod web;

#[tokio::main]
async fn main() {
    libs::logging::Logger::init();

    tracing::info!("Starting server");
    web::server::Server::init().await;
}
