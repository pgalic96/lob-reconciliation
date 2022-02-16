#[cfg(test)]
mod tests {
    pub use crate::order_book::operations;
    use crate::{process_change, process_snapshot, LOBUpdateMessage};
    use std::collections::HashMap;

    const SNAPSHOT_MSG: &str = r#"{
  "params" : {
    "data" : {
      "type" : "snapshot",
      "timestamp" : 1554373962454,
      "instrument_name" : "BTC-PERPETUAL",
      "change_id" : 297217,
      "bids" : [
        [
          "new",
          5042.34,
          30
        ],
        [
          "new",
          5041.94,
          20
        ]
      ],
      "asks" : [
        [
          "new",
          5042.64,
          40
        ],
        [
          "new",
          5043.3,
          40
        ]
      ]
    },
    "channel" : "book.BTC-PERPETUAL.100ms"
  }
}"#;

    const DELETE_MSG: &str = r#"{
    "params" : {
      "data" : {
        "type" : "change",
        "timestamp" : 1554373911330,
        "prev_change_id" : 297217,
        "instrument_name" : "BTC-PERPETUAL",
        "change_id" : 297218,
        "bids" : [
          [
            "delete",
            5041.94,
            0
          ],
          [
            "delete",
            5042.34,
            0
          ]
        ],
        "asks" : [
  
        ]
      },
      "channel" : "book.BTC-PERPETUAL.100ms"
    }
  }"#;

    const NEW_MSG: &str = r#"{
    "params" : {
      "data" : {
        "type" : "change",
        "timestamp" : 1554373911330,
        "prev_change_id" : 297217,
        "instrument_name" : "BTC-PERPETUAL",
        "change_id" : 297218,
        "bids" : [
          [
            "new",
            50423.11,
            12313
          ],
          [
            "new",
            4321143.34,
            123132
          ]
        ],
        "asks" : [
          [
            "new",
            1000.34,
            1231332
          ],
          [
            "new",
            432413,
            1231
          ]
        ]
      },
      "channel" : "book.BTC-PERPETUAL.100ms"
    }
  }"#;

    const CHANGE_MSG: &str = r#"{
    "params" : {
      "data" : {
        "type" : "change",
        "timestamp" : 1554373911330,
        "prev_change_id" : 297217,
        "instrument_name" : "BTC-PERPETUAL",
        "change_id" : 297218,
        "bids" : [
          [
            "change",
            5041.94,
            321312
          ]
        ],
        "asks" : [
          [
            "change",
            5043.3,
            1
          ]
        ]
      },
      "channel" : "book.BTC-PERPETUAL.100ms"
    }
  }"#;

    #[test]
    fn succeeds_processing_snapshot() {
        let mut bid_order_book: HashMap<String, f64> = HashMap::new();
        let mut ask_order_book: HashMap<String, f64> = HashMap::new();
        let msg: LOBUpdateMessage = serde_json::from_str(SNAPSHOT_MSG).unwrap();
        let result = process_snapshot(msg, &mut bid_order_book, &mut ask_order_book);
        assert_eq!(5042.34, result.0);
        assert_eq!(5042.64, result.1);
        assert_eq!(297217, result.2);
        assert_eq!(bid_order_book.into_keys().len(), 2);
        assert_eq!(ask_order_book.into_keys().len(), 2);
    }

    #[test]
    fn succeeds_processing_delete() {
        let mut bid_order_book: HashMap<String, f64> = HashMap::new();
        let mut ask_order_book: HashMap<String, f64> = HashMap::new();
        let msg: LOBUpdateMessage = serde_json::from_str(SNAPSHOT_MSG).unwrap();
        let result = process_snapshot(msg, &mut bid_order_book, &mut ask_order_book);
        let msg: LOBUpdateMessage = serde_json::from_str(DELETE_MSG).unwrap();
        let result = process_change(
            msg,
            &mut bid_order_book,
            &mut ask_order_book,
            result.0,
            result.1,
        );
        assert_eq!(0.0, result.0);
        assert_eq!(5042.64, result.1);
        assert_eq!(297218, result.2);
        assert_eq!(5042.64, result.1);
        assert_eq!(297218, result.2);
        assert_eq!(bid_order_book.into_keys().len(), 0);
        assert_eq!(ask_order_book.into_keys().len(), 2);
    }

    #[test]
    fn succeeds_processing_new() {
        let mut bid_order_book: HashMap<String, f64> = HashMap::new();
        let mut ask_order_book: HashMap<String, f64> = HashMap::new();
        let msg: LOBUpdateMessage = serde_json::from_str(SNAPSHOT_MSG).unwrap();
        let result = process_snapshot(msg, &mut bid_order_book, &mut ask_order_book);
        let msg: LOBUpdateMessage = serde_json::from_str(NEW_MSG).unwrap();
        let result = process_change(
            msg,
            &mut bid_order_book,
            &mut ask_order_book,
            result.0,
            result.1,
        );
        assert_eq!(4321143.34, result.0);
        assert_eq!(1000.34, result.1);
        assert_eq!(297218, result.2);
        assert_eq!(bid_order_book.into_keys().len(), 4);
        assert_eq!(ask_order_book.into_keys().len(), 4);
    }

    #[test]
    fn succeeds_processing_change() {
        let mut bid_order_book: HashMap<String, f64> = HashMap::new();
        let mut ask_order_book: HashMap<String, f64> = HashMap::new();
        let msg: LOBUpdateMessage = serde_json::from_str(SNAPSHOT_MSG).unwrap();
        let result = process_snapshot(msg, &mut bid_order_book, &mut ask_order_book);
        match bid_order_book.get("5041.94") {
            None => panic!("Empty value"),
            Some(i) => assert_eq!(20.0, *i),
        }
        match ask_order_book.get("5043.3") {
            None => panic!("Empty value"),
            Some(i) => assert_eq!(40.0, *i),
        }
        let msg: LOBUpdateMessage = serde_json::from_str(CHANGE_MSG).unwrap();
        let result = process_change(
            msg,
            &mut bid_order_book,
            &mut ask_order_book,
            result.0,
            result.1,
        );
        assert_eq!(5042.34, result.0);
        assert_eq!(5042.64, result.1);
        assert_eq!(297218, result.2);
        match bid_order_book.get("5041.94") {
            None => panic!("Empty value"),
            Some(i) => assert_eq!(321312.0, *i),
        }
        match ask_order_book.get("5043.3") {
            None => panic!("Empty value"),
            Some(i) => assert_eq!(1.0, *i),
        }
        assert_eq!(bid_order_book.into_keys().len(), 2);
        assert_eq!(ask_order_book.into_keys().len(), 2);
    }
}
