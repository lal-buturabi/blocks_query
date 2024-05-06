use serde::Deserialize;
use crate::models::user::UserEvent;
use crate::repository::mongodb_repo::MongoRepo;
use crate::utils::get_all_matched_blocks_with_events;

use actix_web::{
    post,
    get,
    web::{Json, Data, Query},
    HttpResponse,
};

#[derive(Deserialize)]
pub struct AddressQuery {
    addr: String,
}


#[get("/userEvents/")]
pub async fn get_user_blocks_with_events(db: Data<MongoRepo>, Query(addr_q): Query<AddressQuery>) -> HttpResponse {
    let addr = addr_q.addr;
    println!("Got a new request: {}", addr);
    // get all blocks from the db
    // process each block and find if it has any event 
    // which match the given user address
    //
    let matched_blocks = get_all_matched_blocks_with_events(&db, &addr).await;
    
    HttpResponse::Ok().json(matched_blocks)

}

