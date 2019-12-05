extern crate redis;
use redis::{Client,Commands};

// pub fn connexion() -> redis::Connection {  function to create a client and a connexion

// let client = Client::open("redis://127.0.0.1:6379/").unwrap();
// let conn = client.get_connection().unwrap();
// return conn

// }


// pub fn pub_sub(mut conn: redis::Connection){ // function pub/sub redis

//     let mut pubsub = conn.as_pubsub();
//     pubsub.subscribe("channel_1").unwrap();
//     pubsub.subscribe("channel_2").unwrap();

//     loop {
//         let msg = pubsub.get_message().unwrap();
//         let payload : String = msg.get_payload().unwrap();
//         println!("channel '{}': {}", msg.get_channel_name(), payload);
//         break;

// }
// }

  pub fn insert_user(user : String, id_session : String){
   
    let client = Client::open("redis://127.0.0.1:6379/").unwrap();
    let mut conn = client.get_connection().unwrap();
    let _: () = conn.set("user",user) .unwrap();
    let _: () = conn.set("id_session",id_session) .unwrap();
    let answer: String = conn.get("user").unwrap();
    let answer2: String = conn.get("id_session").unwrap();
    println!("USER: {}", answer);
    println!("ID_SESSION : {}",answer2);
  }