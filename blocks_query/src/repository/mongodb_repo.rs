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
        let db_name = match env::var("DATABASE_NAME") {
            Ok(u) => u.to_string(),
            Err(_) => format!("Err loading env var DATABASE_NAME"),
        };
        let col_name = match env::var("COLLECTION_NAME") {
            Ok(u) => u.to_string(),
            Err(_) => format!("Err loading env var COLLECTION_NAME"),
        };
        let client = Client::with_uri_str(&uri).await.unwrap();
        let db = client.database(&db_name);

        let col: Collection<UserEvent> = db.collection(&col_name);
        println!("DB Init Completed.");
        Self { col }
    }


    pub async fn get_all_blocks(&self) -> Result<Cursor<UserEvent>, MongoErr> {
        let filter = doc! {};
        let cursor = self.col.find(filter, None).await;
        cursor
    }
}
