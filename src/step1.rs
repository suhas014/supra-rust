use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;
use tungstenite::connect;
use url::Url;

fn main() {
    let args: Vec<String> = env::args().collect();
    // if args.len() < 3 {
    //     println!("Not enough arguments. Usage: ./simple --mode=<mode> --times=<times>");
    //     return;
    // }
    match args[1].as_str() {
        "--mode=cache" => cache_mode(&args),
        "--mode=read" => read_mode(),
        _ => println!("Invalid command"),
    }
}
fn cache_mode(args: &Vec<String>) {
    let times: u64 = args[2][8..].parse().unwrap();
    let (mut socket, _response) =
        connect(Url::parse("wss://stream.binance.com:9443/ws/btcusdt@trade").unwrap()).expect("Failed to connect");
    // println!("Connected to the server");
    // println!("Response HTTP code: {}", response.status());
    // println!("Response contains the following headers:");
    // for (header, _ /* value */) in response.headers() {
    //     println!("* {}", header);
    // }
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

    let mut file = File::create("data.txt").expect("Unable to create file");
    file.write_all(format!("{}\n{:?}", avg, prices).as_bytes()).expect("Unable to write data");
}

fn read_mode() {
    let mut file = File::open("data.txt").expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read data");
    println!("{}", contents);
}