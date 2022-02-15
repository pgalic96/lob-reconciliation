use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
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
    let mut duration = start.elapsed();
    loop {
        if duration.as_secs() >= 1 {
            output_best_orders(&bid_order_book, &ask_order_book, max_bid, min_ask);
            start = Instant::now();
        }
        let msg = socket.read_message().expect("Error reading message");
        match msg {
            Message::Text(_) => {
                let msg = msg.to_text().expect("Error converting message to text");
                let msg: LOBUpdateMessage = serde_json::from_str(msg).unwrap();
                match msg.params.data.message_type {
                    MessageTypeEnum::Snapshot => {
                        let result =
                            process_snapshot(msg, &mut bid_order_book, &mut ask_order_book);
                        max_bid = result.0;
                        min_ask = result.1;
                        previous_change_id = result.2;
                    }
                    MessageTypeEnum::Change => {
                        if previous_change_id != msg.params.data.prev_change_id {
                            println!("Packet loss -- reconnecting....");
                            bid_order_book.clear();
                            ask_order_book.clear();
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
                        let result = process_change(
                            msg,
                            &mut bid_order_book,
                            &mut ask_order_book,
                            max_bid,
                            min_ask,
                        );
                        if max_bid != result.0 {
                            max_bid = result.0;
                        }
                        if min_ask != result.1 {
                            min_ask = result.1;
                        }
                        previous_change_id = result.2;
                    }
                }
            }
            _ => {
                panic!("Non-text message data received. Aborting.");
            }
        }
        duration = start.elapsed();
    }
}

fn process_snapshot(
    msg: LOBUpdateMessage,
    bid_order_book: &mut HashMap<String, f64>,
    ask_order_book: &mut HashMap<String, f64>,
) -> (f64, f64, u64) {
    let mut max_bid: f64 = 0.0;
    let mut min_ask: f64 = f64::MAX;
    if !msg.params.data.bids.is_empty() {
        max_bid = operations::init_asks(bid_order_book, msg.params.data.bids);
    }
    if !msg.params.data.asks.is_empty() {
        min_ask = operations::init_asks(ask_order_book, msg.params.data.asks);
    }
    (max_bid, min_ask, msg.params.data.change_id)
}

fn process_change(
    msg: LOBUpdateMessage,
    bid_order_book: &mut HashMap<String, f64>,
    ask_order_book: &mut HashMap<String, f64>,
    mut max_bid: f64,
    mut min_ask: f64,
) -> (f64, f64, u64) {
    if !msg.params.data.bids.is_empty() {
        let result = operations::update_bids(bid_order_book, msg.params.data.bids, max_bid);
        if max_bid != result {
            max_bid = result
        }
    }
    if !msg.params.data.asks.is_empty() {
        let result = operations::update_asks(ask_order_book, msg.params.data.asks, min_ask);
        if min_ask != result {
            min_ask = result
        }
    }
    (max_bid, min_ask, msg.params.data.change_id)
}

fn output_best_orders(
    bid_order_book: &HashMap<String, f64>,
    ask_order_book: &HashMap<String, f64>,
    max_bid: f64,
    min_ask: f64,
) {
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
}
