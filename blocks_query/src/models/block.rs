use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;


#[derive(Debug, Serialize, Deserialize)]
pub struct BlocK {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub block_number: u64,
    pub block_hash: String,
    pub events: String,
    pub num_of_events: u32,
}
