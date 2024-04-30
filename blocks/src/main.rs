use mongodb::{options::{ClientOptions, InsertOneOptions, ResolverConfig}, Client};
use std::{env, str::FromStr, sync::Arc};
use std::error::Error;

use mongodb::bson::doc;
use web3::{transports::Http, types::{BlockId, H160, U64}};
use web3::types::{Block, BlockNumber, Transaction};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   let client_uri =
      env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");

   let options =
      ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
         .await?;
   let client = Arc::new(Client::with_options(options)?);


    let http_transport = Http::new("http://3.20.201.137:8545").unwrap();
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
    let mut event_counter: u64 = 0;
    let start_block_height: u64 = 1543162;
    let block_height = web3.eth().block_number().await.unwrap().as_u64();
    let total = block_height - start_block_height;
    let each = total / 1000;

    println!("total blocks to read: {}", total);
    let mut tasks = Vec::new();

    // let logger = Arc::new(RefCell::new(FileLogger::new(INFO_LOG_FILE, ERR_LOG_FILE)?));
    let mut logger = Arc::new(FileLogger::new(INFO_LOG_FILE, ERR_LOG_FILE)?);

    for i in 0..1000u64 {
        let _web3 = web3.clone();
        let _contract = contract.clone();
        let _client = client.clone();
        
        let mut _logger = logger.clone();
        tasks.push(task::spawn(async move {
            process_range(start_block_height + (i * each), (i + 1 + start_block_height) * 800, &_web3, &_contract, &_client, &_logger).await;
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

async fn process_range(start: u64, end: u64, web3: &Web3<Http>, contract: &Contract<Http>, client: &Client, logger: &FileLogger) {
    
    let db = client.database(DB_NAME);
    for bnum in start..=end {
        println!("Block #: {} ", bnum);
        let bnum_str = format!("{}", bnum);

        let block: Block<Transaction> = web3.eth().block_with_txs(BlockId::Number(BlockNumber::Number(U64::from(bnum)))).await.unwrap().unwrap();
        let block_hash = block.hash.unwrap().to_string();
        
        let mut num_of_events_in_a_block = 0;
        let mut num_of_txns_in_a_block = 0;
        let mut event_signatures = Vec::<String>::new();

        for tx in block.transactions {
            num_of_txns_in_a_block +=1 ;
          
            let receipt = web3.eth().transaction_receipt(tx.hash).await.unwrap();
            let logs = receipt.unwrap().logs;
            for _log in logs {
                //println!("log address: {:?}", log.address);
                

                if _log.address == contract.address() {
                    // println!("match with contract");
                    logger.log(LogLevel::Info, "contract matched").await;
                    // let topics = &log.topics;
                    // let data = &log.data;
                    // println!("# of Events: {}", contract.abi().events().map(|_| 1).collect::<Vec<u32>>().len());
                    
                    
                    for event in contract.abi().events() {
                        num_of_events_in_a_block += 1;
                        let expected_signature = format!("{}({})", event.name, event.inputs.iter().map(|i| i.kind.to_string()).collect::<Vec<_>>().join(","));
                        //let log_signature = keccak256(expected_signature.as_bytes());
                        event_signatures.push(expected_signature);
                    }
                }
    
                
            }
            let tx_hash = tx.hash.to_string();
            let tx_doc =  doc! {
                "block_num": bnum_str.as_str(),
                "block_hash": block_hash.as_str(),
                "txn_hash": tx_hash,
            };
            let h =  db.collection(TXN_COLLECTION).insert_one(tx_doc, InsertOneOptions::default()).await;
            if h.is_err() {
                logger.log(LogLevel::Err, format!("{:#?}", h.err()).as_str()).await
            }
        }

        let blk_doc =  doc! {
            "block_num": bnum_str,
            "block_hash": block_hash,
            "num_of_transactions": num_of_txns_in_a_block,
            "num_of_events": num_of_events_in_a_block,
            "event_signatures": event_signatures.join("::").as_str(),
        };
        let h =  db.collection(BLOCK_COLLECTION).insert_one(blk_doc, InsertOneOptions::default()).await;
        
        if h.is_err() {
            logger.log(LogLevel::Err, &format!("Error writing into {}: {:#?}", BLOCK_COLLECTION, h.err())).await
        }
        logger.log(
            LogLevel::Info, 
            &format!(
                "Block Num: {}\nNum of Txns: {}\nNum of Events: {}", 
                bnum, 
                num_of_txns_in_a_block, 
                num_of_events_in_a_block
            )
        ).await;
        
    }
}