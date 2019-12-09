extern crate redis;
use redis::{Client,Commands};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};




// function to create a client and a connexion
// pub fn connexion() -> redis::Connection {  

// let client = Client::open("redis://127.0.0.1:6379/").unwrap();
// let conn = client.get_connection().unwrap();
// return conn

// }

  pub fn hash() {
      let user ="TEAM TAPASS";
      let id_session = "58816e6f-0afc-43dd-8899-67fa05db895b";
      let mut hasher = DefaultHasher::new();
      user.hash(&mut hasher);
      id_session.hash(&mut hasher);
      println!("Hash is {:x}!", hasher.finish());
  }


  pub fn insert_user_and_session(user : String, id_session: &str) {
   
    let client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
    let mut con = client.get_connection().unwrap();
    redis::cmd("HSET").arg(user).arg("session").arg(id_session).execute(&mut con);
    println!("Enregistrer en BDD");
  }

  pub fn get_user(user : String) -> String{
    return user;
  } 

  pub fn get_session (id_session : &str)-> &str{
    return id_session;
  }