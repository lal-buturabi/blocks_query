// use mongodb::{options::{ClientOptions, InsertOneOptions, ResolverConfig}, Client};
use std::{collections::{HashMap, HashSet}, env, hash::Hash, str::FromStr, sync::Arc};
use std::error::Error;

use mongodb::bson::{doc, Document};
use web3::{signing::keccak256, transports::Http, types::{BlockId, Filter, FilterBuilder, H160, H256, U64}};
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

// use crate::logger::Logger;
use hex::encode;
// use regex::Regex;
use ethabi::{ParamType, decode};

// DB related constants
// const DB_NAME: &str = "Nexa_Diagnostics";
// const TXN_COLLECTION: &str = "txns_table";
// const EVENT_COLLECTION: &str = "events_table";
// const BLOCK_COLLECTION: &str = "blocks_table";

// log file path constants
const ERR_LOG_FILE: &str = "./error.log";
const INFO_LOG_FILE: &str = "./info.log";
// const DEBUG_LOG_FILE: &str = "./debug.log";
// const WARN_LOG_FILE: &str = "./warning.log";

// struct CustomHashSet<T>(Vec<>)


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
//    let client_uri =
//       env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
   let rpc_url =
      env::var("RPC_URL3").expect("You must set the RPC_URL environment var!");
   let abi_path =
      env::var("ABI_PATH").expect("You must set the RPC_URL environment var!");

//    let options =
//       ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
//          .await?;
//    let client = Arc::new(Client::with_options(options)?);


    let http_transport = Http::new(&rpc_url).unwrap();
    let web3 = Arc::new(Web3::new(http_transport));

    let mut file = File::open(abi_path).expect("Failed to open ABI file");
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
    
    let ev_sighashs = Arc::new(
        contract.abi().events().into_iter().map(|event| { 
            let sig = format!("{}({})", event.name, event.inputs.iter().map(|i| i.kind.to_string()).collect::<Vec<_>>().join(","));
            let index_str = event.inputs.iter().map(|ip| if ip.indexed { "1" } else { "0" }).collect::<Vec<_>>().join(",");
            let h = keccak256(sig.as_bytes());
            println!("{}\t\t\t{:?}\n{}\n", sig, encode(h), index_str);
            
            (h, (sig, index_str))
        }).collect::<HashMap<[u8; 32], (String, String)>>()
    );
    println!("\n\n");
    //let start_block_height: usize = 1543162;
    let start_block_height: usize = 1801212;
    let block_height = web3.eth().block_number().await.unwrap().as_usize();
    let total = block_height - start_block_height;
    let each = total / 1000;

    println!("height: {}\nstart: {}\neach: {}\ntotal: {}", block_height, start_block_height, each, total);
    // return Ok(());
    let mut tasks = Vec::new();

    // let logger = Arc::new(RefCell::new(FileLogger::new(INFO_LOG_FILE, ERR_LOG_FILE)?));
    let mut logger = Arc::new(FileLogger::new(INFO_LOG_FILE, ERR_LOG_FILE)?);

    for _ in 0..1usize {
        let _web3 = web3.clone();
        let _contract = contract.clone();
        // let _client = client.clone();
        let _ev_sighashs = ev_sighashs.clone();
        
        let mut _logger = logger.clone();
        tasks.push(task::spawn(async move {
            process_range(
                start_block_height, 
                start_block_height, 
                &_web3, 
                &_contract, 
                &_ev_sighashs, 
                &_logger
            ).await;
        }));
    }

    web3::block_on(async {
        for task in tasks {
            let _ = task.await;
        }
    });

    println!("Done!");

   Ok(())
}


fn to_param_types(sig: &str, idx: &str) -> (Vec<ParamType>, Vec<ParamType>) {
    let mut substr = String::new();
    let mut s = false;
    for c in sig.chars() {
        if c == '(' {
            s = true;
            continue;
            // m = true;
        }
        if c == ')' {
            s = false
        }
        if s {
            substr.push(c)
        }
    }
    println!("event signature: {}", substr);
    let idxs = idx.split(',');
    // println!("IDXs: {:?}", idxs);
    let clz = |(i, s)| {
        match s {
            "address" => ParamType::Address,
            "bytes" => ParamType::Bytes,
            "uint8" => ParamType::Uint(8),
            "uint16" => ParamType::Uint(16),
            "uint20" => ParamType::Uint(20),
            "uint24" => ParamType::Uint(24),
            "uint32" => ParamType::Uint(32),
            "uint40" => ParamType::Uint(40),
            "uint128" => ParamType::Uint(128),
            "uint256" => ParamType::Uint(256),
            "bool" => ParamType::Bool,
            "string" => ParamType::String,
            _ => ParamType::Int(256)
        }
    };
    let i_params: Vec<&str> = substr.split(',').zip(idxs.clone()).filter(|(_, c)| if c == &"1" {true} else {false}).map(|s| s.0).collect();
    let ni_params: Vec<&str> = substr.split(',').zip(idxs).filter(|(_, c)| if c == &"0" {true} else {false}).map(|s| s.0).collect();
    // println!("i_params: {:#?}\nni_params: {:#?}", i_params, ni_params);
    let iparam_types: Vec<ParamType> = i_params.into_iter().enumerate().map(clz).collect();
    let niparam_types: Vec<ParamType> = ni_params.into_iter().enumerate().map(clz).collect();
    (iparam_types, niparam_types)

}

async fn decode_logs(
    caddr: &H160,
    logs: Vec<Log>, 
    ev_sighashs: &HashMap<[u8; 32], (String, String)>) -> HashMap<H256, Vec<String>> {
    
    let mut block_to_evts_map = HashMap::<H256, Vec<String>>::new();
    let mut matched_evts: Vec<String> = Vec::new();
    let mut matched_data_vec = Vec::<String>::new();
    let len = logs.len();

    for mut i in 0..len {
        let log = logs.get(i).unwrap().clone();
        println!("log address: {:?}", log.address);             
        let mut event_data_str = String::new();
        // let mut matched_data = Vec::<String>::new();
        if &log.address == caddr {
            let topics = log.topics;
            let data = log.data;
            // println!("Index: {:?}", log.log_index);
            println!("Data: {:#?}", data);
            
            let sig = topics[0].as_bytes();
            if let Some(v) = ev_sighashs.get(sig) {
                // push event signature
                event_data_str.push_str(&v.0);

                let (i_ptypes, ni_ptypes) = to_param_types(&v.0, &v.1);
                
                // will only decode non-indexed params
                if ni_ptypes.len() > 0 {
                    let decoded = decode(&ni_ptypes, data.0.as_ref());
                    
                    if decoded.is_ok() {
                        println!("decoded: {:?}", decoded);
                        // that means we have some indexed params here, so look into topics
                        // push non-indexed params
                        event_data_str.push_str(&format!(":NonIndexed({})", decoded.unwrap().into_iter().map(|token| format!("0x{}", token)).collect::<Vec<String>>().join(",")));
                    }
                }

                if i_ptypes.len() > 0 {
                
                    println!("[O] Topics: {:#?}", topics);
                    let t: String = topics.into_iter().skip(1).map(|topic| {
                        let s = format!("{:?}", topic).trim_start_matches(|c| c == '0' || c == 'x').to_owned();
                        if s.is_empty() {
                            "0x0".to_owned()
                        } else {
                            format!("0x{}", s)
                        }
                    }).collect::<Vec<_>>().join(",");
                    println!("Topics: {:?}", t);
                    // finally push Indexed params
                    event_data_str.push_str(&format!(":Indexed({})", t));
                }
                
                // println!("index: {:#?}\nnon-index: {:#?}", i_ptypes, ni_ptypes);
                // matched_data.push()
            }
        }
        // put the string into the vec under its block hash key in the map
        // no  matter what order are the logs in wrt block hash
        block_to_evts_map.entry(log.block_hash.unwrap())
            .or_insert(Vec::new())
            .push(event_data_str);
    }

    //println!("decoded data: {:#?}", event_data_str);
   block_to_evts_map 
}

// fn zero_trim(s: &[u8]) -> String {
    // println!("trimming.. {:?}", s);
    // let mut s: Vec<_> = s.into_iter().skip(2).collect();
    // let mut i = 0;
    // while s.get(i).unwrap() == &&0 {
        // s.remove(i);
        // i += 1;
    // }
    // let s = s.iter().map(|byte| format!("{:02x}", byte)).collect();
    // println!("trimmed: {}", s);
    // s
// }

async fn process_range(start: usize, end: usize, web3: &Web3<Http>, contract: &Contract<Http>, ev_sighashs: &HashMap<[u8; 32],  (String, String)>, logger: &FileLogger) {
    
    // let db = client.database(DB_NAME);
    println!("start: {}, end: {}", start, end);
    println!("current block: {}", web3.eth().block_number().await.unwrap());
    println!("chainId: {}", web3.eth().chain_id().await.unwrap());
    println!("protocol_version: {}", web3.eth().protocol_version().await.unwrap());
    
    for bnum in start..=end {
        let bn_start = BlockNumber::Number(U64::from(bnum));
        let bn_end = bn_start.clone();
        println!("Block #: {} ", bnum);
        
        let logFilter = FilterBuilder::default().address(vec![contract.address()]).from_block(bn_start).to_block(bn_end).build();
        let logs: Vec<Log> = web3.eth().logs(logFilter).await.unwrap();
        
        println!("got logs: {}", logs.len());
        let logs_decoded = decode_logs(&contract.address(), logs, ev_sighashs).await;
        
        // let mut num_of_events_in_a_block = 0;
        // let mut num_of_txns_in_a_block = 0;
        // let mut event_signatures = Vec::<String>::new();
        // let blocks: Vec<BlockModel> = Vec::with_capacity(end - start + 1);

           // num_of_txns_in_a_block +=1 ;
          
            
            // let tx_hash = tx.hash.to_string();
            // let block_doc =  doc! {
            //     "events": logs_decoded, 
            //     // "block_num": bnum_str.as_str(),
            //     //"block_hash": block_hash.as_str(),
            // };
            let documents: Vec<Document> = logs_decoded.iter().map(|(block_hash, events)| {
                let events_string = events.join("::");
                
                doc! {
                    "block_hash": format!("{:?}", block_hash),
                    "events": events_string,
                    "num_of_events": events.len() as i32,
                }
            }).collect();   
            println!("Document: {:#?}", documents);
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
