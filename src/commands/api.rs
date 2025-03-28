pub async fn run(bind: &String) {
    crate::libs::config::Config::new_empty();
    crate::web::server::Server::init(bind).await;
}
