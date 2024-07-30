
use bson::doc;
use crate::models::user::UserEvent;
use crate::repository::mongodb_repo::MongoRepo;

use actix_web::{web::Data, HttpResponse};
use hex::decode;

use std::io::Read;
use futures::future::join_all;
use flate2::read::ZlibDecoder;
use serde::{Serialize, Deserialize};
use futures_util::stream::TryStreamExt;
use async_std::sync::Mutex;
use std::sync::Arc;



#[derive(Debug, Serialize, Deserialize)]
pub struct Res {
    total_matched: usize,
    total_blocks: usize,
    top: UserEvent,
}

impl Res {
    pub fn new(m: &usize, b: &usize, top: &UserEvent) -> Self {
        Self {
            total_matched: m.to_owned(),
            total_blocks: b.to_owned(),
            top: top.clone(),
        }
    }
}


pub async fn get_all_matched_blocks_with_events(db: Arc<Data<MongoRepo>>, addr: &str) -> Vec<Res> {
    let addr_arc = Arc::new(addr.to_lowercase()); 
    let matched_blocks_mx: Arc<Mutex<Vec<UserEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let t = Arc::new(Mutex::new(0));
    let mt = Arc::new(Mutex::new(0));

    let num_batches = 1000;
    let batch_size = 800; // Adjust this value based on your document count

    // Create tasks to fetch documents in parallel
    let mut tasks = Vec::new();
    
    for batch_num in 0..num_batches {
        let start = batch_num * batch_size;
        let end = (batch_num + 1) * batch_size;
        let mb_mx = Arc::clone(&matched_blocks_mx);
        let addr = Arc::clone(&addr_arc);
        let dba = Arc::clone(&db);
        let ta = Arc::clone(&t);
        let mta = Arc::clone(&mt);

        let task = async move {
            if let Ok(mut cursor) = dba.get_all_blocks(start, end).await {
                
                while let Some(mut block) = cursor.try_next().await.unwrap() {
                    //let mut block: UserEvent = bson::from_document(doc.clone()).unwrap();
                    let mut total = ta.lock().await;
                    *total += 1;
                    drop(total);
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
                            if event.to_lowercase().contains(&*addr) {
                                Some(event.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();
                        println!("matched events len: {}", matched_events.len());
                    if matched_events.len() > 0 {
                        let matched_events_str = matched_events.join("::");

                        block.events = matched_events_str;
                        block.num_of_events = matched_events.len() as u32;
                        
                        let mut mtotal = mta.lock().await;
                        *mtotal += 1;
                        drop(mtotal);
                        let mut matched_blocks = mb_mx.lock().await;
                        matched_blocks.push(block);
                        drop(matched_blocks);
                    }
                }
            }
        };
        tasks.push(task);
    }
    let _ = join_all(tasks).await;
    let total = t.lock().await;
    let mtotal = mt.lock().await;
    println!("total: {} mtotal: {}", total, mtotal);

    let mblocks = Arc::clone(&matched_blocks_mx);
    let matched_blocks = mblocks.lock().await;
    let u = UserEvent::default();
    vec![Res::new(&matched_blocks.len(), &total, if matched_blocks.len() > 0 {matched_blocks.get(0).unwrap()} else { &u})]

}


fn decompress_it(d: &[u8]) -> Result<String, std::io::Error> {
    let mut dec = ZlibDecoder::new(d);
    let mut s = String::new();
    dec.read_to_string(&mut s)?;
    Ok(s)
}
