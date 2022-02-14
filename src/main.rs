use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tungstenite::{connect, Message};
use url::Url;

mod order_book;

pub use crate::order_book::operations;

const SUBSCRIBE_MSG: &str = r#"
{
    "jsonrpc" : "2.0",
    "method" : "public/subscribe",
    "params" : {
        "channels": ["book.BTC-PERPETUAL.100ms"]
    }
}
"#;

const UNSUBSCRIBE_MSG: &str = r#"
        {
            "jsonrpc" : "2.0",
            "method" : "public/unsubscribe",
            "params" : {
                "channels": ["book.BTC-PERPETUAL.100ms"]
            }
        }
    "#;


#[derive(Debug, Deserialize, Serialize)]
struct LOBUpdateMessage {
    params: Data,
}

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    data: LOBMessage,
}

#[derive(Debug, Deserialize, Serialize)]
struct LOBMessage {
    #[serde(rename = "type")]
    message_type: MessageTypeEnum,
    #[serde(default)]
    prev_change_id: u64,
    change_id: u64,
    bids: Vec<operations::PriceUpdate>,
    asks: Vec<operations::PriceUpdate>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum MessageTypeEnum {
    Change,
    Snapshot,
}

fn main() {
    let (mut socket, _) =
        connect(Url::parse("wss://www.deribit.com/ws/api/v2").unwrap()).expect("Can't connect");

    socket
        .write_message(Message::Text(SUBSCRIBE_MSG.into()))
        .unwrap();
    socket.read_message().expect("Error reading message");
    let mut previous_change_id: u64 = 0;
    let mut max_bid: f64 = 0.0;
    let mut min_ask: f64 = f64::MAX;
    let mut bid_order_book: HashMap<String, f64> = HashMap::new();
    let mut ask_order_book: HashMap<String, f64> = HashMap::new();
    let mut start = Instant::now();

    loop {
        let duration = start.elapsed();
        if duration >= Duration::new(1, 0) {
            if max_bid != 0.0 {
                let amount = bid_order_book.get(&max_bid.to_string());
                match amount {
                    None => panic!("There cannot be a price with empty amount"),
                    Some(i) => {
                        println!("Best Bid: {}, Amount: {:?}", max_bid, *i as i64);
                    }
                }
            }
            if min_ask != f64::MAX {
                let amount = ask_order_book.get(&min_ask.to_string());
                match amount {
                    None => panic!("There cannot be a price with empty amount"),
                    Some(i) => {
                        println!("Best Ask: {}, Amount: {:?}", min_ask, *i as i64);
                        println!("-------------------");
                    }
                }
            }
            start = Instant::now();
        }
        let msg = socket.read_message().expect("Error reading message");
        match msg {
            Message::Text(_) => {
                let msg = match msg.to_text() {
                    Ok(val) => val,
                    Err(_) => continue,
                };
                let msg: LOBUpdateMessage = serde_json::from_str(msg).unwrap();
                match msg.params.data.message_type {
                    MessageTypeEnum::Snapshot => {
                        if msg.params.data.bids.len() != 0 {
                            let result = operations::init_asks(
                                &mut bid_order_book,
                                msg.params.data.bids,
                            );
                            max_bid = result;
                        }
                        if msg.params.data.asks.len() != 0 {
                            let result = operations::init_asks(
                                &mut ask_order_book,
                                msg.params.data.asks,
                            );
                            min_ask = result;
                        }
                        previous_change_id = msg.params.data.change_id;
                    }
                    MessageTypeEnum::Change => {
                        if previous_change_id != msg.params.data.prev_change_id {
                            println!("Packet loss -- reconnecting....");
                            bid_order_book = HashMap::new();
                            ask_order_book = HashMap::new();
                            socket
                                .write_message(Message::Text(UNSUBSCRIBE_MSG.into()))
                                .unwrap();
                            socket.read_message().expect("Error reading message");
                            socket
                                .write_message(Message::Text(SUBSCRIBE_MSG.into()))
                                .unwrap();
                            socket.read_message().expect("Error reading message");
                            continue;
                        }
                        previous_change_id = msg.params.data.change_id;
                        if msg.params.data.bids.len() != 0 {
                            let result = operations::update_bids(
                                &mut bid_order_book,
                                msg.params.data.bids,
                                max_bid,
                            );
                            if max_bid != result {
                                max_bid = result
                            }
                        }
                        if msg.params.data.asks.len() != 0 {
                            let result = operations::update_asks(
                                &mut ask_order_book,
                                msg.params.data.asks,
                                min_ask,
                            );
                            if min_ask != result {
                                min_ask = result
                            }
                        }
                    }
                }
            }
            _ => {
                panic!("Non-text message data received. Aborting.");
            }
        }
    }
}
