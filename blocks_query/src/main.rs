mod api;
mod utils;
mod models;
mod repository;

use std::sync::Arc;
use env_logger::Builder;
use actix_web::{web::Data, App, HttpServer};
use repository::mongodb_repo::MongoRepo;
use api::user_api::get_user_blocks_with_events;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    Builder::new()
        // .filter_module("actix_web", log::LevelFilter::Debug)
        .filter(None, log::LevelFilter::Debug)
        .init();

    let db = MongoRepo::init().await;
    // let data_arc = Arc::new(Data::new(db));
    let data = Data::new(db);

    println!("Listening on localhost:8080");
    HttpServer::new(move || {
        // let data = Arc::clone(&data_arc);
        App::new()
        .app_data(data.clone())
        .service(get_user_blocks_with_events)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
