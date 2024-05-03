use mongodb::{options::{ClientOptions, InsertManyOptions, ResolverConfig}, Client};
use std::{collections::HashMap, env, str::FromStr, sync::Arc};
use std::error::Error;

use flate2::{write::ZlibEncoder, read::ZlibDecoder, Compression};
use std::io::{Write, Read};
use mongodb::bson::{doc, Document};
use web3::{signing::keccak256, transports::Http, types::{FilterBuilder, H160, H256, U64}};
use web3::types::{Log, BlockNumber};
use web3::Web3;
use web3::contract::Contract;

use serde_json::Value;
use std::fs::File;
use tokio::task;

use tokio;

mod logger;
use logger::{LogLevel, FileLogger};

use crate::logger::Logger;
use hex::encode;
// use regex::Regex;
use ethabi::{ParamType, decode};
use async_std::sync::Mutex;
use bson::{Bson};
// DB related constants
const DB_NAME: &str = "Nexa_Events_Data_2";
// const TXN_COLLECTION: &str = "txns_table";
const EVT_COLLECTION: &str = "events_table";
// const BLOCK_COLLECTION: &str = "blocks_table";

// log file path constants
const ERR_LOG_FILE: &str = "./error.log";
const INFO_LOG_FILE: &str = "./info.log";
// const DEBUG_LOG_FILE: &str = "./debug.log";
// const WARN_LOG_FILE: &str = "./warning.log";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   let client_uri =
      env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
   let rpc_url =
      env::var("RPC_URL3").expect("You must set the RPC_URL environment var!");
   let abi_path =
      env::var("ABI_PATH").expect("You must set the RPC_URL environment var!");

   let options =
      ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
         .await?;
   let client = Arc::new(Client::with_options(options)?);


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
    
    let ev_sighashs = Arc::new(Mutex::new(
        contract.abi().events().into_iter().map(|event| { 
            let sig = format!("{}({})", event.name, event.inputs.iter().map(|i| i.kind.to_string()).collect::<Vec<_>>().join(","));
            let index_str = event.inputs.iter().map(|ip| if ip.indexed { "1" } else { "0" }).collect::<Vec<_>>().join(",");
            let h = keccak256(sig.as_bytes());
            println!("{}\t\t\t{:?}\n{}\n", sig, encode(h), index_str);
            
            (h, (sig, index_str))
        }).collect::<HashMap<[u8; 32], (String, String)>>()
    ));
    println!("\n\n");
    let start_block_height: usize = 1543162;
    //let start_block_height: usize = 1801212;
    //let end_block_height: usize = 1801214;
    let num_of_batches = 10000;
    let block_height = web3.eth().block_number().await.unwrap().as_usize();
    let total = block_height - start_block_height;
    let batch_size = total / num_of_batches;
    let mut failed_batches = Arc::new(Mutex::new(Vec::<(usize, usize)>::new()));
    
    // println!("height: {}\nstart: {}\neach: {}\ntotal: {}", block_height, start_block_height, each, total);
    // return Ok(());
    let mut tasks = Vec::new();

    //let logger = Arc::new(RefCell::new(FileLogger::new(INFO_LOG_FILE, ERR_LOG_FILE)?));
    let mut logger = Arc::new(FileLogger::new(INFO_LOG_FILE, ERR_LOG_FILE)?);

    for i in 0..num_of_batches {
        let _web3 = Arc::clone(&web3);
        let _contract = Arc::clone(&contract);
        let _client = Arc::clone(&client);
        let _failed_batches = Arc::clone(&mut failed_batches);
        let _ev_sighashs = Arc::clone(&ev_sighashs);
        let mut _logger = Arc::clone(&logger);
        
        let bstart = start_block_height + i * batch_size;
        let bend = if i == num_of_batches - 1 {
            block_height
        } else {
            start_block_height  + (i + 1) * batch_size - 1
        };

        tasks.push(task::spawn(async move {
            process_range(
                bstart, 
                bend, 
                _web3, 
                _contract, 
                _failed_batches,
                _ev_sighashs, 
                _logger,
                _client
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
    // println!("event signature: {}", substr);
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
    ev_sighashs: &HashMap<[u8; 32], (String, String)>) -> (HashMap<H256, Vec<String>>, HashMap<H256, i32>) {
    
    let mut block_to_evts_map = HashMap::<H256, Vec<String>>::new();
    let mut bnum_map = HashMap::<H256, i32>::new();
    let mut matched_evts: Vec<String> = Vec::new();
    let mut matched_data_vec = Vec::<String>::new();
    let len = logs.len();

    for mut i in 0..len {
        let log = logs.get(i).unwrap().clone();
        
        // println!("log address: {:?}", log.address);             
        let mut event_data_str = String::new();
        // let mut matched_data = Vec::<String>::new();
        if &log.address == caddr {
            let topics = log.topics;
            let data = log.data;
            // println!("Index: {:?}", log.log_index);
            // println!("Data: {:#?}", data);
            
            let sig = topics[0].as_bytes();
            if let Some(v) = ev_sighashs.get(sig) {
                // push event signature
                event_data_str.push_str(&v.0);

                let (i_ptypes, ni_ptypes) = to_param_types(&v.0, &v.1);
                
                // will only decode non-indexed params
                if ni_ptypes.len() > 0 {
                    let decoded = decode(&ni_ptypes, data.0.as_ref());
                    
                    if decoded.is_ok() {
                        // println!("decoded: {:?}", decoded);
                        // that means we have some indexed params here, so look into topics
                        // push non-indexed params
                        event_data_str.push_str(&format!(":NonIndexed({})", decoded.unwrap().into_iter().map(|token| format!("0x{}", token)).collect::<Vec<String>>().join(",")));
                    }
                }

                if i_ptypes.len() > 0 {
                
                    // println!("[O] Topics: {:#?}", topics);
                    let t: String = topics.into_iter().skip(1).map(|topic| {
                        let s = format!("{:?}", topic).trim_start_matches(|c| c == '0' || c == 'x').to_owned();
                        if s.is_empty() {
                            "0x0".to_owned()
                        } else {
                            format!("0x{}", s)
                        }
                    }).collect::<Vec<_>>().join(",");
                    // println!("Topics: {:?}", t);
                    // finally push Indexed params
                    event_data_str.push_str(&format!(":Indexed({})", t));
                }
                
                // println!("index: {:#?}\nnon-index: {:#?}", i_ptypes, ni_ptypes);
                // matched_data.push()
            }
        }
        // put the string into the vec under its block hash key in the map
        // no  matter what order are the logs in wrt block hash
        let bh = log.block_hash.unwrap();
        let bn = log.block_number.unwrap().as_u32() as i32;
        bnum_map.entry(bh)
            .or_insert(bn);
        block_to_evts_map.entry(bh)
            .or_insert(Vec::new())
            .push(event_data_str);
    }

    //println!("decoded data: {:#?}", event_data_str);
    (block_to_evts_map, bnum_map) 
}

fn compress_it(s: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    
    enc.write_all(s.as_bytes())?;
    enc.finish()
}

fn decompress_it(d: &[u8]) -> Result<String, std::io::Error> {
    let mut dec = ZlibDecoder::new(d);
    let mut s = String::new();
    dec.read_to_string(&mut s)?;
    Ok(s)
}

async fn process_range(
    start: usize, 
    end: usize, 
    web3: Arc<Web3<Http>>, 
    contract: Arc<Contract<Http>>, 
    failed_batches: Arc<Mutex<Vec<(usize, usize)>>>,
    ev_sighashs: Arc<Mutex<HashMap<[u8; 32],  (String, String)>>>, 
    logger: Arc<FileLogger>,
    client: Arc<Client>
) {
    
    let db = client.database(DB_NAME);

    let bn_start = BlockNumber::Number(U64::from(start));
    let bn_end = BlockNumber::Number(U64::from(end));
    
    let logFilter = FilterBuilder::default().address(vec![contract.address()]).from_block(bn_start).to_block(bn_end).build();
    let logs_res = web3.eth().logs(logFilter).await;
    // let logs: Vec<Log> = web3.eth().logs(logFilter).await;
    if logs_res.is_err() {
        // wait for the lock
        let mut fb = failed_batches.lock().await;
        fb.push((start.clone(), end.clone()));
        logger.log(LogLevel::Err, &format!("Log Fetch Failure ({}, {})", start, end));
        return;
    }

    let logs = logs_res.unwrap();
    
    println!("got logs: {}", logs.len());
    let mut num_of_events = 0;
    
    // wait for the lock
    let mut sighashes = ev_sighashs.lock().await;

    let (logs_decoded, bnum_map) = decode_logs(&contract.address(), logs, &sighashes).await;

    let documents: Vec<Document> = logs_decoded.iter().map(|(block_hash, events)| {
        let joined = events.join("::");
        let events_string = encode(compress_it(&joined).unwrap());
        // println!("compressed: {} decompressed: {} original: {}", events_string.len(),decompress_it(&hex::decode(&events_string).unwrap()).unwrap().len(), joined.len());
        // println!("Decompressed: {}", decompress_it(&hex::decode(&events_string).unwrap()).unwrap());
        
        num_of_events += events.len();

        doc! {
            "block_number":bnum_map.get(&block_hash).unwrap(),
            "block_hash": format!("{:?}", block_hash),
            "events": events_string,
            "num_of_events": events.len() as i32,
        }
    }).collect();   
    println!("Document: {:#?}", documents);
    let h =  db.collection(EVT_COLLECTION).insert_many(documents, InsertManyOptions::default()).await;
    if h.is_err() {
        logger.log(LogLevel::Err, format!("{:#?}", h.err()).as_str()).await
    } else {
        logger.log(
            LogLevel::Info, 
            &format!(
                "Inserted Block numbers from: {}\nTo: {}\nNum of Events: {}", 
                start, 
                end, 
                num_of_events 
            )
        ).await;
    }
}
