mod api;
mod models;
mod repository;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Blok {
    
}

#[get("/")]
async fn home() -> impl Responder {
    HttpResponse::Ok().json("thank u for using this app!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Listening on localhost:8080");
    HttpServer::new(|| App::new().service(home))
            .bind(("localhost", 8080))?
            .run()
            .await
}
