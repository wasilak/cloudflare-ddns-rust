pub async fn run(path: &String, bind: &String) {
    crate::libs::config::Config::load_from_yaml(path).unwrap();
    crate::web::server::Server::init(&bind).await;
}
