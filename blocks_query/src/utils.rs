
use bson::doc;
use crate::models::user::UserEvent;
use crate::repository::mongodb_repo::MongoRepo;

use actix_web::{web::Data, HttpResponse};
use hex::decode;

use std::io::Read;
use flate2::read::ZlibDecoder;
use futures_util::stream::TryStreamExt;

pub async fn get_all_matched_blocks_with_events(db: &Data<MongoRepo>, addr: &str) -> Vec<UserEvent> {
    let addr = addr.to_lowercase(); 
    let mut matched_blocks: Vec<UserEvent> = Vec::new();
    if let Ok(mut cursor) = db.get_all_blocks().await {

        while let Some(mut block) = cursor.try_next().await.unwrap() {
            //let mut block: UserEvent = bson::from_document(doc.clone()).unwrap();

            let events = match decode(&block.events) {
                Ok(bytes) => {
                    decompress_it(&bytes).unwrap()
                }
                Err(_) => {
                    continue; // which is v v rare
                }
            };

            let splits: Vec<&str> = events.split("::").collect();

            let matched_events: Vec<String> = splits
                .iter()
                .filter_map(|event| {
                    if event.to_lowercase().contains(&addr) {
                        Some(event.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            let matched_events_str = matched_events.join("::");

            block.events = matched_events_str;

            matched_blocks.push(block);
        }
    }
    matched_blocks
}


fn decompress_it(d: &[u8]) -> Result<String, std::io::Error> {
    let mut dec = ZlibDecoder::new(d);
    let mut s = String::new();
    dec.read_to_string(&mut s)?;
    Ok(s)
}
