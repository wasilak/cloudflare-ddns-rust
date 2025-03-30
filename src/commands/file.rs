pub async fn run(path: &String, bind: &String, refresh_interval: u64) {
    crate::libs::config::Config::load_from_yaml(path).unwrap();
    crate::libs::runner::refresh_dns_loop(refresh_interval).await;
    crate::web::server::Server::init(&bind).await;
}
