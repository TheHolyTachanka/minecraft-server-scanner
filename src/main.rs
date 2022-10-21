/* #![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::correctness,
    clippy::complexity,
    clippy::perf,
    clippy::restriction
)] */

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
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("{}", "[ERROR] No ip list specified!".red());
        exit(1);
    } else if args.len() > 2 {
        println!("{}", "[ERROR] Too many arguments!".red());
        exit(1);
    }
    if args.contains(&"-h".to_owned()) || args.contains(&"--help".to_owned()) {
        println!("\nUsage:");
        println!("mc-scanner <ip-list-file>");
    } else {
        let ip_list_string = if let Ok(i) = fs::read_to_string(&args[1]) {
            i
        } else {
            println!("{}", "[ERROR] Ip list file doesnt exist!".red());
            exit(1);
        };
        let ip_vec: Vec<String> = ip_list_string
            .split('\n')
            .map(|s| String::from(s))
            .collect();

        // This could probably be made multi-threaded, but I'm too lazy.
        for ip in ip_vec {
            let json = get_server_info(&ip);
            add_to_db(&json, &ip);
        }
    }
}
/// I used the mcstatus python library because I couldn't get the rust version to work.
fn get_server_info(ip: &str) -> String {
    let output = Command::new("python")
        .args(["-m", "mcstatus", ip, "json"])
        .output()
        .expect("failed to execute process");

    let json = if let Ok(i) = String::from_utf8(output.stdout) {
        i
    } else {
        println!("{}", "[ERROR] Failed to read STDOUD".red());
        exit(1);
    };

    println!("[INFO] Got response from {}", ip);
    json
}

fn add_to_db(json: &str, ip: &str) {
    let client = if let Ok(i) = Client::with_uri_str(if let Ok(i) = env::var("MONGODB_URL") {
        i
    } else {
        println!("{}", "[ERROR] Could not connect to DB!".red());
        exit(1);
    }) {
        i
    } else {
        println!("{}", "[ERROR] Could not connect to DB!".red());
        exit(1);
    };
    let database = client.database("Servers");
    let collection = database.collection::<Server>("Servers");

    let docs = vec![Server {
        ip: ip.to_string(),
        json: json.to_owned(),
    }];

    if !json.contains("\"online\": false") && !json.is_empty() {
        println!("{}", format!("[INFO] {} is online!", ip).green());
        if let Ok(i) = collection.insert_many(docs, None) {
            i
        } else {
            println!("{}", "[ERROR] Could not insert server in DB!".red());
            exit(1);
        };
        println!("{}", format!("[INFO] Added {} to DB", ip).green());
    } else {
        println!("{}", format!("[WARN] {} is offline!", ip).red());
    }
}
