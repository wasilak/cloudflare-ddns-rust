pub async fn run(bind: &String, refresh_interval: u64) {
    crate::libs::config::Config::new_empty();
    crate::libs::runner::refresh_dns_loop(refresh_interval).await;
    crate::web::server::Server::init(bind).await;
}
