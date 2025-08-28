use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
pub mod handlers;
pub mod routes;
pub mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // if let Err(e) =
    //     rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider())
    // {
    //     eprintln!("Failed to install CryptoProvider: {:?}", e);
    //     std::process::exit(1);
    // }
    dotenv().ok();
    env_logger::init();
    let s3_region = std::env::var("AWS_REGION").expect("AWS_REGION is required");
    let s3_endpoint = std::env::var("AWS_S3_ENDPOINT").expect("AWS_S3_ENDPOINT is required");
    let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .endpoint_url(s3_endpoint)
        .region(aws_sdk_s3::config::Region::new(s3_region))
        .load()
        .await;
    let s3_client = aws_sdk_s3::Client::new(&aws_config);
    println!("🚀 Server starting");

    HttpServer::new(move || {
        let app = App::new().app_data(web::Data::new(s3_client.clone()));
        app.configure(routes::configure_routes)
    })
    .workers(8)
    .backlog(4096)
    .bind(std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:7860".into()))?
    .run()
    .await
}
