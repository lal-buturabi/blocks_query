use mongodb::{options::{ClientOptions, InsertOneOptions, ResolverConfig}, Client};
use std::{collections::{HashMap, HashSet}, env, hash::Hash, str::FromStr, sync::Arc};
use std::error::Error;

use mongodb::bson::doc;
use web3::{signing::keccak256, transports::Http, types::{BlockId, Filter, FilterBuilder, H160, U64}};
use web3::types::{Log, BlockNumber, Transaction};
use web3::Web3;
use web3::contract::Contract;

use serde_json::Value;
use std::fs::File;
use std::io::Read;
use tokio::task;

use tokio;

mod logger;
use logger::{LogLevel, FileLogger};

use crate::logger::Logger;
use hex::encode;

// DB related constants
const DB_NAME: &str = "Nexa_Diagnostics";
const TXN_COLLECTION: &str = "txns_table";
const EVENT_COLLECTION: &str = "events_table";
const BLOCK_COLLECTION: &str = "blocks_table";

// log file path constants
const ERR_LOG_FILE: &str = "./logs/error.log";
const INFO_LOG_FILE: &str = "./logs/info.log";
const DEBUG_LOG_FILE: &str = "./logs/debug.log";
const WARN_LOG_FILE: &str = "./logs/warning.log";

// struct CustomHashSet<T>(Vec<>)


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
//    let client_uri =
//       env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
   let rpc_url =
      env::var("RPC_URL3").expect("You must set the RPC_URL environment var!");

//    let options =
//       ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
//          .await?;
//    let client = Arc::new(Client::with_options(options)?);


    let http_transport = Http::new(&rpc_url).unwrap();
    let web3 = Arc::new(Web3::new(http_transport));

    let mut file = File::open("./src/abi.json").expect("Failed to open ABI file");
    let mut abi_string = String::new();
    file.read_to_string(&mut abi_string).expect("Failed to read ABI file");

    let abi_json: Value = serde_json::from_str(&abi_string).expect("Failed to parse ABI JSON");
    let abi_json_string = serde_json::to_string(&abi_json).expect("Failed to serialize ABI JSON");
    // println!("ABI JSON Value: {}", abi_json);

    let contract_address = H160::from_str("0xDB54D3Ce2035509d83F86bc982adc62F4AEBe03c")?;
    let contract_abi = abi_json_string.as_bytes();
    let contract = Arc::new(Contract::from_json(
        web3.eth(),
        contract_address,
        contract_abi,
    )?);
  
    
    let event_list = Arc::new(
        contract.abi().events().into_iter().map(|event| { 
            let sig = format!("{}({})", event.name, event.inputs.iter().map(|i| i.kind.to_string()).collect::<Vec<_>>().join(","));
            let hash = keccak256(sig.as_bytes());
            println!("sign: {}, hash: {:?}", sig, encode(hash));
            (hash, sig)
        }).collect::<HashMap<[u8; 32], String>>());
    return Ok(());
    let start_block_height: usize = 1801212;
    let block_height = web3.eth().block_number().await.unwrap().as_usize();
    let total = block_height - start_block_height;
    let each = total / 1000;

    println!("total blocks to read: {}", total);
    let mut tasks = Vec::new();

    // let logger = Arc::new(RefCell::new(FileLogger::new(INFO_LOG_FILE, ERR_LOG_FILE)?));
    let mut logger = Arc::new(FileLogger::new(INFO_LOG_FILE, ERR_LOG_FILE)?);

    for i in 0..1usize {
        let _web3 = web3.clone();
        let _contract = contract.clone();
        // let _client = client.clone();
        let _event_list = event_list.clone();
        
        let mut _logger = logger.clone();
        tasks.push(task::spawn(async move {
            process_range(start_block_height, start_block_height, &_web3, &_contract, &_event_list, &_logger).await;
        }));
    }

    web3::block_on(async {
        for mut task in tasks {
            task.await;
        }
    });

    println!("Done!");

   Ok(())
}

#[derive(Debug, Default)]
struct BlockModel {
    hash: String,
    number: usize,
    number_of_events: u16,

}

async fn process_range(start: usize, end: usize, web3: &Web3<Http>, contract: &Contract<Http>, event_list: &HashMap<[u8; 32], String>, logger: &FileLogger) {
    
    // let db = client.database(DB_NAME);
    println!("start: {}, end: {}", start, end);
    println!("current block: {}", web3.eth().block_number().await.unwrap());
    println!("chainId: {}", web3.eth().chain_id().await.unwrap());
    println!("protocol_version: {}", web3.eth().protocol_version().await.unwrap());
    
    for bnum in start..=end {
        let bn_start = BlockNumber::Number(U64::from(bnum));
        let bn_end = bn_start.clone();
        println!("Block #: {} ", bnum);
        // let bnum_str = format!("{}", bnum);
        let eventFilter = FilterBuilder::default().address(vec![contract.address()]).from_block(bn_start).to_block(bn_end).build();
        let logs: Vec<Log> = web3.eth().logs(eventFilter).await.unwrap();
       // let block_hash = block.hash.unwrap().to_string();
       println!("got logs: {}", logs.len());
        
        // let mut num_of_events_in_a_block = 0;
        // let mut num_of_txns_in_a_block = 0;
        // let mut event_signatures = Vec::<String>::new();
        // let blocks: Vec<BlockModel> = Vec::with_capacity(end - start + 1);

           // num_of_txns_in_a_block +=1 ;
          
            for _log in logs {
                println!("log address: {:?}", _log.address);             
                let mut matched_events = Vec::<String>::new();
                if _log.address == contract.address() {
                    //blocks
                    // println!("match with contract");
                    logger.log(LogLevel::Info, "contract matched").await;
                    // let topics = &log.topics;
                    // let data = &log.data;
                    // println!("# of Events: {}", contract.abi().events().map(|_| 1).collect::<Vec<u32>>().len());
                    let bs = _log.data.0.bytes();
                    
                    // _log.topics.clone().into_iter().for_each(|topic| {
                    //     if let Some(v) = event_list.get(topic.as_bytes()) {
                    //         matched_events.push(v.to_owned());
                    //     }
                    // });
                }
                // println!("Topics #: {}, Events #: {}\nViz: \n\t{:?}", _log.topics.len(), matched_events.len(), matched_events);
                println!("Topics #: {}, Data Len: {}\nData: \n\t{:?}", _log.topics.len(), _log.data.0.len(), _log.data.0.chunks_exact(20));
                
            }
            // let tx_hash = tx.hash.to_string();
            // let tx_doc =  doc! {
            //     "block_num": bnum_str.as_str(),
            //     "block_hash": block_hash.as_str(),
            //     "txn_hash": tx_hash,
            // };
            // let h =  db.collection(TXN_COLLECTION).insert_one(tx_doc, InsertOneOptions::default()).await;
            // if h.is_err() {
            //     logger.log(LogLevel::Err, format!("{:#?}", h.err()).as_str()).await
            // }
        

        // let blk_doc =  doc! {
        //     "block_num": bnum_str,
        //     "block_hash": block_hash,
        //     "num_of_transactions": num_of_txns_in_a_block,
        //     "num_of_events": num_of_events_in_a_block,
        //     "event_signatures": event_signatures.join("::").as_str(),
        // };
        // let h =  db.collection(BLOCK_COLLECTION).insert_one(blk_doc, InsertOneOptions::default()).await;
        
        // if h.is_err() {
        //     logger.log(LogLevel::Err, &format!("Error writing into {}: {:#?}", BLOCK_COLLECTION, h.err())).await
        // }
        // logger.log(
        //     LogLevel::Info, 
        //     &format!(
        //         "Block Num: {}\nNum of Txns: {}\nNum of Events: {}", 
        //         bnum, 
        //         num_of_txns_in_a_block, 
        //         num_of_events_in_a_block
        //     )
        // ).await;
        
    }
}
