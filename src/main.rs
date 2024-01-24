use ring::{signature::{self, Ed25519KeyPair, KeyPair}, rand};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;
use tungstenite::connect;
use url::Url;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use base64::{encode, decode};

fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "--mode=cache" => cache_mode(&args),
        "--mode=read" => read_mode(),
        "--mode=distributed" => distributed_mode(),
        _ => println!("Invalid command"),
    }
}

fn cache_mode(args: &Vec<String>) {
    let rng = rand::SystemRandom::new();
    let key_pair = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
    let key_pair = Ed25519KeyPair::from_pkcs8(key_pair.as_ref()).unwrap();

    let times: u64 = args[2][8..].parse().unwrap();
    let (mut socket, _response) =
        connect(Url::parse("wss://stream.binance.com:9443/ws/btcusdt@trade").unwrap()).expect("Failed to connect");
    let mut prices = vec![];
    for _i in 0..times {
        let msg = socket.read_message().expect("Error reading message");
        let v: serde_json::Value = serde_json::from_str(msg.into_text().unwrap().as_str()).unwrap();
        let price: f64 = v["p"].as_str().unwrap().parse().unwrap();
        prices.push(price);
        std::thread::sleep(Duration::from_secs(1));
    }
    let sum: f64 = prices.iter().sum();
    let avg = sum / (prices.len() as f64);
    println!("Cache complete. The average USD price of BTC is: {}", avg);

    let signature = key_pair.sign(avg.to_string().as_bytes());
    let public_key = key_pair.public_key().as_ref(); // Get the public key bytes

    // Encode the signature and public key as base64
    let signature_base64 = encode(signature.as_ref());
    let public_key_base64 = encode(public_key);

    let mut file = File::create("data.txt").expect("Unable to create file");
    file.write_all(format!("{}\n{:?}\n{}\n{}", avg, prices, signature_base64, public_key_base64).as_bytes()).expect("Unable to write data");
}

fn read_mode() {
    let mut file = File::open("data.txt").expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read data");

    let lines: Vec<&str> = contents.split('\n').collect();
    let avg = lines[0];
    let signature_base64 = lines[2];
    let public_key_base64 = lines[3]; // Read the public key bytes from the file

    // Decode the base64 signature and public key
    let signature = decode(signature_base64).expect("Failed to decode base64 signature");
    let public_key_bytes = decode(public_key_base64).expect("Failed to decode base64 public key");

    let public_key = signature::UnparsedPublicKey::new(&signature::ED25519, &public_key_bytes);

    match public_key.verify(avg.as_bytes(), &signature) {
        Ok(()) => println!("The signature is valid."),
        Err(_) => println!("The signature is not valid."),
    }
    file.read_to_string(&mut contents).expect("Unable to read data");
    println!("{}", contents);
}
fn distributed_mode() {
    let (tx, rx): (Sender<f64>, Receiver<f64>) = channel();
    let mut handles = vec![];

    for _ in 0..5 {
        let thread_tx = tx.clone();
        let handle = thread::spawn(move || {
            let avg = cache_and_compute_avg();
            thread_tx.send(avg).unwrap();
        });
        handles.push(handle);
    }

    let mut avgs = vec![];
    for _ in 0..5 {
        avgs.push(rx.recv().unwrap());
    }

    let sum: f64 = avgs.iter().sum();
    let avg = sum / (avgs.len() as f64);
    println!("Aggregation complete. The average of averages USD price of BTC is: {}", avg);
}

fn cache_and_compute_avg() -> f64 {
    let rng = rand::SystemRandom::new();
    let key_pair = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
    let _key_pair = Ed25519KeyPair::from_pkcs8(key_pair.as_ref()).unwrap();

    let (mut socket, _response) =
        connect(Url::parse("wss://stream.binance.com:9443/ws/btcusdt@trade").unwrap()).expect("Failed to connect");
    let mut prices = vec![];
    for _i in 0..10 {
        let msg = socket.read_message().expect("Error reading message");
        let v: serde_json::Value = serde_json::from_str(msg.into_text().unwrap().as_str()).unwrap();
        let price: f64 = v["p"].as_str().unwrap().parse().unwrap();
        prices.push(price);
        std::thread::sleep(Duration::from_secs(1));
    }
    let sum: f64 = prices.iter().sum();
    let avg = sum / (prices.len() as f64);
    avg
}