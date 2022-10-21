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

use dialoguer::{theme::ColorfulTheme, Confirm};
use rand::Rng;
use colored::Colorize;
use mongodb::{bson::doc, sync::Client};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::process::exit;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct Server {
    _id: String,
    json: String,
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let mongo_url = match env::var("MONGODB_URL") {
        Ok(i) => i,
        Err(_) => {
            println!("{}", "[ERROR] $MONGODB_URL not set!".red());
            exit(1);
        }
    };

    let client = match Client::with_uri_str(mongo_url) {
        Ok(i) => i,
        Err(_) => {
            println!("[ERROR] Could not connect to DB");
            exit(1);
        }
    };


    if args.len() == 1 {
        println!("{}", "[ERROR] No ip list specified!".red());
        exit(1);
    } else if args.len() > 2 {
        println!("{}", "[ERROR] Too many arguments!".red());
        exit(1);
    }
    if args.contains(&"-h".to_owned()) || args.contains(&"--help".to_owned()) {
        println!("\nUsage:");
        println!("Dictionary example:");
        println!("mc-scanner <ip-list-file>");
        println!("Brute-force example:");
        println!("mc-scanner -b");
    }else if args[1] == "-b" {
        if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("This will run forever, stop it with CTRL+C, do you want to continue?(y/n)")
        .interact()
        .unwrap()
        {
            loop {
                let ip = gen_ipv4();
                let json = get_server_info(&ip);
                add_to_db(&json, &ip, &client);
            }
        } else {
            println!("{}", "[QUIT] Canceled by user!".red())
        }
    }else {
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
            add_to_db(&json, &ip, &client);
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

    println!("[INFO] Getting response from {}", ip);
    json
}

fn add_to_db(json: &str, ip: &str, client: &Client) {
    //create_server_index_1(&client);
    let database = client.database("Servers");
    let collection = database.collection::<Server>("Servers");

    let doc = Server {
        _id: ip.to_string(),
        json: json.to_owned(),
    };

    if !json.contains("\"online\": false") && !json.is_empty() {
        println!("{}", format!("[INFO] {} is online!", ip).green());
        if let Ok(i) = collection.insert_one(doc, None) {
            i
        } else {
            println!("{} {} {}", "[ERROR]".red(), ip.red(), "Was a duplicate!".red());
            exit(1);
        };
        println!("{}", format!("[INFO] Added {} to DB", ip).green());
    } else {
        println!("{}", format!("[WARN] {} is offline!", ip).red());
    }
}

fn gen_ipv4() -> String {
    let mut rnd = rand::thread_rng();
    let a = rnd.gen_range(1..255);
    let b = rnd.gen_range(1..255);
    let c = rnd.gen_range(1..255);
    let d = rnd.gen_range(1..255);

    return format!("{}.{}.{}.{}", a, b, c, d);
}