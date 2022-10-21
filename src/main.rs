#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
)]

extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use colored::Colorize;
use mongodb::{bson::doc, sync::Client};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::process::exit;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct Server {
    ip: String,
    json: String,
}
/// Some docs
fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    if &args[1] == "-h" || &args[1] == "--help" {
        println!("Usage:");
        println!("mc-scanner <ip-list-file>");
    } else {
        let ip_list_string = match fs::read_to_string(&args[1]) {
            Ok(i) => i,
            Err(_) => {
                println!("{}", "[ERROR] Ip list file doesnt exist!".red());
                exit(1);
            }
        };
        let ip_vec: Vec<String> = ip_list_string.split('\n').map(|s| s.to_string()).collect();

        // This could probably be made multi-threaded, but I'm too lazy.
        for ip in ip_vec {
            let json = get_server_info(ip.to_string());
            add_to_db(json, ip.to_string());
        }
    }
}
/// I used the mcstatus python library because I couldn't get the rust version to work.
fn get_server_info(ip: String) -> String {
    let output = Command::new("python")
        .args(["-m", "mcstatus", &ip, "json"])
        .output()
        .expect("failed to execute process");

    let json = String::from_utf8(output.stdout).unwrap();
    println!("[INFO] Got response from {}", ip);
    json
}

fn add_to_db(json: String, ip: String) {
    let client =
        Client::with_uri_str(env::var("MONGODB_URL").expect("$MONGODB_URL is not set")).unwrap();
    let database = client.database("Servers");
    let collection = database.collection::<Server>("Servers");

    let docs = vec![Server {
        ip: ip.clone(),
        json: json.clone(),
    }];

    if !json.contains("\"online\": false") && !json.is_empty() {
        println!("{}", format!("[INFO] {} is online!", ip).green());
        collection.insert_many(docs, None).unwrap();
        println!("{}", format!("[INFO] Added {} to DB", ip).green());
    } else {
        println!("{}", format!("[WARN] {} is offline!", ip).red());
    }
}
