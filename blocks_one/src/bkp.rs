



// use chrono::{TimeZone, Utc};
// use serde::{Deserialize, Serialize};
// use mongodb::bson::{Bson, oid::ObjectId};
// use std::collections::HashMap;
// use web3::api::Eth;
// use web3::signing::keccak256;
// #[derive(Serialize, Deserialize, Debug)]
// struct BlockDetail {
//    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
//    id: Option<ObjectId>,
//    title: String,
//    year: i32,
//    plot: String,
//    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
//    released: chrono::DateTime<Utc>,
// }

// let new_doc = doc! {
//     "title": "Parasite",
//     "year": 2020,
//     "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
//     "released": Utc.ymd(2020, 2, 7).and_hms(0, 0, 0),
//  };
//    println!("Databases:");
//    for name in client.list_database_names(None, None).await? {
//       println!("- {}", name);
//    }


//  println!("{}", new_doc);

    // let movies = client.database("sample_mflix").collection("movies");
    // let insert_result = movies.insert_one(new_doc.clone(), None).await?;
    // println!("New document ID: {}", insert_result.inserted_id);

    // let movie = movies
    // .find_one(
    //     doc! {
    //             "title": "Parasite"
    //     },
    //     None,
    // ).await?
    // .expect("Missing 'Parasite' document.");
    // println!("Movie: {}", movie);

    // let captain_marvel = Movie {
    //     id: None,
    //     title: "Captain Marvel".to_owned(),
    //     year: 2019,
    //     plot: "".to_owned(),
    //     released: Utc.ymd(2010, 7, 16).and_hms(0, 0, 0)
    // };
    

    // let serialized_movie = bson::to_bson(&captain_marvel)?;
    // let document = serialized_movie.as_document().unwrap();
    // println!("{}", document);    
                      
                      
                      
                        // if topics.get(0) == Some(&web3::types::H256(log_signature)) {

                            // Decode indexed and non-indexed parameters
                            // let indexed_params = Log::parse_topics(topics, &event.inputs[..event.inputs.iter().filter(|i| i.indexed).count()])?;
                            // let non_indexed_params = Log::parse_data(data, &event.inputs[event.inputs.iter().filter(|i| i.indexed).count()..])?;

                            // // Combine and access parameters
                            // let mut all_params = indexed_params.into_iter().zip(non_indexed_params).collect::<Vec<_>>();
                            // let mut decoded_params = Vec::new();
                            let mut offset = 0;
                            println!("Event: {}", event.name);
                            // for input in event.inputs.iter() {
                            //     // println!("kind: {:?}, name: {:?},  indexed: {}", input.kind, input.name, input.indexed);
                            //     // let decoded_param = match input.kind.to_string().as_str() {
                            //     //     "address" => Value::from(format!("{:?}", H256::from_slice(input..0.bytes()[..]))),
                            //     //     "uint256" => U256::from_slice(&data[offset..offset + 32]),
                            //     //     "int256" => U256::from_slice(&data[offset..offset + 32]),
                            //     //     // ... (adapt for other types)
                            //     //     _ => Value::from(&data[offset..offset + input.type_.len()]), // Decode as raw bytes
                            //     // };
                            //     // offset += input.type_.len();
                            //     // decoded_params.push(decoded_param);
                            // }
                           
                            // for (name, value) in event.inputs.iter().enumerate() {
                            //     println!("  - {}: {:?}", name, all_params.get_mut(name).unwrap());
                            // }

                            // break; // Stop iterating events if a match is found
                        // }