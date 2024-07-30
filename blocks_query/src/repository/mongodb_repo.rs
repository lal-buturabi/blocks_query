use std::env;

use dotenv::dotenv;

use crate::models::user::UserEvent;
use bson::doc;
use mongodb::{Collection, Client, Cursor, error::Error as MongoErr};

#[derive(Debug)]
pub struct MongoRepo {

    col: Collection<UserEvent>,
}

impl MongoRepo {
    pub async fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGODB_URI") {
            Ok(u) => u.to_string(),
            Err(_) => format!("Err loading env var MONGO_URL"),
        };
        println!("URI: {}", uri);
        let db_name = match env::var("DATABASE_NAME") {
            Ok(u) => u.to_string(),
            Err(_) => format!("Err loading env var DATABASE_NAME"),
        };
        println!("DB: {}", db_name);
        let col_name = match env::var("COLLECTION_NAME") {
            Ok(u) => u.to_string(),
            Err(_) => format!("Err loading env var COLLECTION_NAME"),
        };
        println!("Collection: {}", col_name);
        let client = Client::with_uri_str(&uri).await.unwrap();
        let db = client.database(&db_name);

        let col: Collection<UserEvent> = db.collection(&col_name);
        println!("DB Init Completed.");
        Self { col }
    }


    pub async fn get_all_blocks(&self, start: usize, end: usize) -> Result<Cursor<UserEvent>, MongoErr> {
        let filter = doc! { "block_number": { "$gte": start as u32, "$lt": end as u32 } };
        let cursor = self.col.find(filter, None).await;
        println!("range: ({}, {})", start, end);
        cursor
    }
}
