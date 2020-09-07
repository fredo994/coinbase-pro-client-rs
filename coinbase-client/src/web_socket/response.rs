use std::collections::HashMap;

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor, Error};
use serde::export::Formatter;
use serde::ser::SerializeSeq;
use serde_json::Value;

use super::common::Channel;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Side { BUY, SELL }

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum OrderType { LIMIT, MARKET, STOP }

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum FinishReason { FILLED, CANCELED }

// @formatter:off
#[serde(tag = "type", rename_all = "lowercase")]
#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseMessages {
  Subscriptions { #[serde(flatten)] resp: SubscriptionResponse },
  Heartbeat     { #[serde(flatten)] resp: HeartBeatResponse    },
  Status        { #[serde(flatten)] resp: StatusResponse       },
  Ticker        { #[serde(flatten)] resp: TickerResponse       },
  Snapshot      { #[serde(flatten)] resp: SnapshotResponse     },
  L2Update      { #[serde(flatten)] resp: L2UpdateResponse     },
  Match         { #[serde(flatten)] resp: MatchResponse        },
  Received      { #[serde(flatten)] resp: ReceivedResponse     },
  Open          { #[serde(flatten)] resp: OpenResponse         },
  Change        { #[serde(flatten)] resp: ChangeResponse       },
  Done          { #[serde(flatten)] resp: DoneResponse         },
  Active        { #[serde(flatten)] resp: ActiveResponse       },
  Error         { #[serde(flatten)] resp: ErrorResponse        },
  // Note that since we are converting tag to lowercase,
  // then the we force the snake_case here instead of camelCase.
  #[allow(non_camel_case_types)]
  Last_Match    { #[serde(flatten)] resp: LastMatchResponse    },
}
// @formatter:on

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscriptionResponse {
  pub channels: Vec<Channel>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HeartBeatResponse {
  pub sequence: i64,
  pub last_trade_id: i64,
  pub product_id: String,
  pub time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusResponse {
  pub products: Vec<Product>,
  pub currencies: Vec<Currency>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TickerResponse {
  pub trade_id: i64,
  pub sequence: i64,
  pub time: DateTime<Utc>,
  pub product_id: String,
  pub price: BigDecimal,
  pub side: Side,
  pub last_size: BigDecimal,
  pub best_bid: BigDecimal,
  pub best_ask: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SnapshotResponse {
  // TODO sequence number or not?
  pub product_id: String,
  pub bids: Vec<Vec<BigDecimal>>,
  pub asks: Vec<Vec<BigDecimal>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct L2UpdateResponse {
  // TODO sequence number or maybe there is no need for sequence number since l2update maybe in order
  //  always.
  pub product_id: String,
  pub time: DateTime<Utc>,
  pub changes: Vec<Change>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MatchResponse {
  pub time: DateTime<Utc>,
  pub product_id: String,
  pub sequence: i64,
  pub trade_id: i64,
  pub maker_order_id: String,
  pub taker_order_id: String,
  pub size: BigDecimal,
  pub price: BigDecimal,
  pub side: Side,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReceivedResponse {
  pub time: DateTime<Utc>,
  pub product_id: String,
  pub sequence: i64,
  pub order_id: String,
  pub side: Side,
  pub order_type: OrderType,

  // For limit orders
  pub size: Option<BigDecimal>,
  pub price: Option<BigDecimal>,

  // For Market orders
  pub funds: Option<BigDecimal>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenResponse {
  pub time: DateTime<Utc>,
  pub product_id: String,
  pub sequence: i64,
  pub order_id: String,
  pub price: BigDecimal,
  pub side: Side,
  pub remaining_size: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangeResponse {
  pub time: DateTime<Utc>,
  pub product_id: String,
  pub sequence: i64,
  pub order_id: String,
  pub new_size: BigDecimal,
  pub old_size: BigDecimal,
  pub price: Option<BigDecimal>,
  pub side: Side,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DoneResponse {
  pub time: DateTime<Utc>,
  pub product_id: String,
  pub sequence: i64,
  pub order_id: String,
  pub reason: FinishReason,
  pub side: Side,
}

// TODO consider if this is necessary.
#[derive(Serialize, Deserialize, Debug)]
pub struct ActiveResponse {
  pub time: DateTime<Utc>,
  pub product_id: String,
  pub order_id: String,
  pub user_id: String,
  pub profile_id: String,
  pub timestamp: String,
  // Not really sure what this is
  pub stop_type: String,
  pub side: Side,
  pub stop_price: BigDecimal,
  pub size: BigDecimal,
  pub funds: BigDecimal,
  pub private: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LastMatchResponse {
  pub trade_id: i64,
  pub maker_order_id: String,
  pub taker_order_id: String,
  pub side: Side,
  pub size: BigDecimal,
  pub price: BigDecimal,
  pub product_id: String,
  pub sequence: i64,
  pub time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
  pub msg: String,
  pub extra: HashMap<String, Value>,
}

/////////////////////////////
// Product                 //
/////////////////////////////

#[derive(Serialize, Deserialize, Debug)]
pub struct Product {
  id: String,
  base_currency: String,
  quote_currency: String,
  base_min_size: Option<BigDecimal>,
  base_max_size: Option<BigDecimal>,
  base_increment: Option<BigDecimal>,
  quote_increment: Option<BigDecimal>,
  display_name: String,
  status: Option<String>,
  status_message: Option<String>,
  min_market_funds: Option<BigDecimal>,
  max_market_funds: Option<BigDecimal>,
  post_only: bool,
  limit_only: bool,
  cancel_only: Option<bool>,
}

/////////////////////////////
// Currency                //
/////////////////////////////

#[derive(Serialize, Deserialize, Debug)]
pub struct Currency {
  id: String,
  name: String,
  min_size: BigDecimal,
  status: String,
  status_message: Option<String>,
  max_precision: BigDecimal,
  convertible_to: Vec<String>,
}

/////////////////////////////
// Change                  //
/////////////////////////////

#[derive(Debug)]
pub struct Change {
  side: Side,
  price: BigDecimal,
  size: BigDecimal,
}

impl Serialize for Change {
  fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
    S: Serializer {
    let mut serializer = serializer.serialize_seq(Some(3))?;
    serializer.serialize_element(&self.side)?;
    serializer.serialize_element(&self.price)?;
    serializer.serialize_element(&self.size)?;
    serializer.end()
  }
}

impl<'de> Deserialize<'de> for Change {
  fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
    D: Deserializer<'de> {
    struct ChangeVisitor;
    impl<'de> Visitor<'de> for ChangeVisitor {
      type Value = Change;

      fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("struct Change")
      }

      fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, <A as SeqAccess<'de>>::Error> where
        A: SeqAccess<'de>, {
        let side = seq.next_element()?
          .ok_or_else(|| Error::invalid_length(0, &self))?;
        let price = seq.next_element()?
          .ok_or_else(|| Error::invalid_length(1, &self))?;
        let size = seq.next_element()?
          .ok_or_else(|| Error::invalid_length(2, &self))?;
        Ok(Change { side, price, size })
      }
    }
    deserializer.deserialize_seq(ChangeVisitor)
  }
}


#[cfg(test)]
mod test {
  use serde_json;

  use super::ResponseMessages;

  #[test]
  fn deserialize_heartbeat_msg() -> Result<(), serde_json::error::Error> {
    let json = r#"
      {
          "type": "heartbeat",
          "sequence": 90,
          "last_trade_id": 20,
          "product_id": "BTC-USD",
          "time": "2014-11-07T08:19:28.464459Z"
      }
    "#;
    match serde_json::from_str(json)? {
      ResponseMessages::Heartbeat { resp } => {
        println!("Ok got: {:?}", resp);
      }
      _ => {
        assert!(false)
      }
    };
    return Ok(());
  }

  #[test]
  fn test_ticker_deserialization() -> Result<(), serde_json::error::Error> {
    let msg = r#"
      {
        "type":"ticker",
        "product_id":"ETH-USD",
        "sequence":10182181199,
        "price":"432.39",
        "trade_id":62994234,
        "side":"sell",
        "time":"2020-08-31T14:37:46.082020Z",
        "last_size":"16.18415683",
        "best_bid":"432.39",
        "best_ask":"432.47"
      }
    "#;
    let ticker: ResponseMessages = serde_json::from_str(msg)?;
    match ticker {
      ResponseMessages::Ticker { resp: _ } => {},
      _ => assert!(false)
    };
    Ok(())
  }

  #[test]
  fn test_match_deserialize() -> Result<(), serde_json::error::Error> {
    let msg = r#"{
    "type":"done",
    "side":"buy",
    "product_id":"ETH-USD",
    "time":"2020-08-31T14:55:23.850342Z",
    "sequence":10182302147,
    "order_id":"478e4673-4ad5-4138-b943-c168081f7e4a",
    "reason":"canceled",
    "price":"434.1",
    "remaining_size":"0.63324286"
    }"#;
    let done = serde_json::from_str(msg)?;
    match done {
      ResponseMessages::Done { resp: _ } => {},
      _ => {
        assert!(false);
      }
    };
    return Ok(());
  }

  #[test]
  fn test_status_deserialize() -> Result<(), serde_json::error::Error> {
    let msg = r#"
    {
    "type":"status",
    "currencies":[
      {
        "id":"ALGO",
        "name":"Algorand",
        "min_size":"1.00000000",
        "status":"online",
        "funding_account_id":"1b4aa4bd-47cd-4197-8218-a1f597ebaef8",
        "status_message":"",
        "max_precision":"0.0000010000000000000000000000000000000000",
        "convertible_to":[],
        "details": {
          "type":"crypto",
          "symbol":"A",
          "network_confirmations":1,
          "sort_order":93,
          "crypto_address_link":"https://algoexplorer.io/address/{{address}}",
          "crypto_transaction_link":"https://algoexplorer.io/tx/{{txId}}",
          "push_payment_methods":["crypto"],
          "processing_time_seconds":5,
          "min_withdrawal_amount":0.1
          }
        },
        {
         "id":"DASH",
         "name":"Dash",
         "min_size":"1.00000000",
         "status":"online",
         "funding_account_id":"2cc59af1-d6cd-4726-991f-9ecd2da9f98a",
         "status_message":"",
         "max_precision":"0.0000000100000000000000000000000000000000",
         "convertible_to":[],
         "details": {
            "type":"crypto",
            "symbol":"",
            "network_confirmations":2,
            "sort_order":47,
            "crypto_address_link":"https://chain.so/address/DASH/{{address}}",
            "crypto_transaction_link":"https://chain.so/tx/DASH/{{address}}",
            "push_payment_methods":["crypto"],"min_withdrawal_amount":0.01
         }
        },
        {
         "id":"OXT",
         "name":"Orchid",
         "min_size":"1.00000000",
         "status":"online",
         "funding_account_id":"875263aa-82dd-470a-85cf-518306f63ee6",
         "status_message":"",
         "max_precision":"0.0000000100000000000000000000000000000000",
         "convertible_to":[],
         "details":{"type":"crypto","symbol":"","network_confirmations":35,"sort_order":48,"crypto_address_link":"https://etherscan.io/token/0x4575f41308EC1483f3d399aa9a2826d74Da13Deb?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}
         },
         {"id":"ATOM","name":"Cosmos","min_size":"1.00000000","status":"online","funding_account_id":"2176d8be-bf5f-4a8c-a86d-92ca9512ddb5","status_message":"","max_precision":"0.0000010000000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"","network_confirmations":0,"sort_order":51,"crypto_address_link":"https://cosmos.bigdipper.live/account/{{address}}","crypto_transaction_link":"https://cosmos.bigdipper.live/transactions/{{txId}}","push_payment_methods":["crypto"],"processing_time_seconds":5,"min_withdrawal_amount":0.1}},{"id":"KNC","name":"Kyber Network","min_size":"1.00000000","status":"online","funding_account_id":"312ff600-7dbb-453c-8719-95beb94adaf4","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Ξ","network_confirmations":35,"sort_order":120,"crypto_address_link":"https://etherscan.io/token/0xdd974d5c2e2928dea5f71b9825b8b646686bd200?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"XRP","name":"XRP","min_size":"1.00000000","status":"online","funding_account_id":"37c2cccd-0cf9-4d64-87d8-2b2b3d205a41","status_message":"","max_precision":"0.0000010000000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"$","network_confirmations":0,"sort_order":30,"crypto_address_link":"https://bithomp.com/explorer/{{address}}","crypto_transaction_link":"https://bithomp.com/explorer/{{txId}}","push_payment_methods":["crypto"],"processing_time_seconds":600,"min_withdrawal_amount":22}},{"id":"REP","name":"Augur","min_size":"0.00000100","status":"online","funding_account_id":"57aff206-3abf-4b8e-8aa2-0054ddf877c5","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Ξ","network_confirmations":35,"sort_order":85,"crypto_address_link":"https://etherscan.io/token/0x1985365e9f78359a9B6AD760e32412f4a445E862?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"MKR","name":"Maker","min_size":"0.00100000","status":"online","funding_account_id":"9b5effc5-eef4-4c80-9712-980f464a6642","status_message":"","max_precision":"0.0001000000000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"","network_confirmations":35,"sort_order":49,"crypto_address_link":"https://etherscan.io/token/0x9f8f72aa9304c8b593d555f12ef6589cc3a579a2?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"COMP","name":"Compound","min_size":"0.01000000","status":"online","funding_account_id":"039cca5b-563c-467d-9a23-a01c8145232d","status_message":"","max_precision":"0.0010000000000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"","network_confirmations":35,"sort_order":140,"crypto_address_link":"https://etherscan.io/token/0xc00e94cb662c3520282e6f5717214004a7f26888?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"NMR","name":"Numeraire","min_size":"0.01000000","status":"online","funding_account_id":"7be34fa5-dfe5-46da-8981-6af0a2e355e7","status_message":"","max_precision":"0.0010000000000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"","network_confirmations":35,"sort_order":170,"crypto_address_link":"https://etherscan.io/token/0x1776e1F26f98b1A5dF9cD347953a26dd3Cb46671?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"OMG","name":"OMG Network","min_size":"1.00000000","status":"online","funding_account_id":"cc86b83b-32a8-43ae-988f-46eb9b0f774a","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"","network_confirmations":35,"sort_order":57,"crypto_address_link":"https://etherscan.io/token/0xd26114cd6EE289AccF82350c8d8487fedB8A0C07?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"BAND","name":"Band Protocol","min_size":"0.10000000","status":"online","funding_account_id":"b7627141-16af-4c3c-9775-1bbf9b14c91b","status_message":"","max_precision":"0.0100000000000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"","network_confirmations":35,"sort_order":160,"crypto_address_link":"https://etherscan.io/token/0xba11d00c5f74255f56a5e366f4f77f5a186d7f55?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"XLM","name":"Stellar","min_size":"1.00000000","status":"online","funding_account_id":"00516bcd-a7c7-4198-afef-d64ed641c6af","status_message":"","max_precision":"0.0000001000000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"","network_confirmations":0,"sort_order":50,"crypto_address_link":"https://stellar.expert/explorer/public/account/{{address}}","crypto_transaction_link":"https://stellar.expert/explorer/public/tx/{{txId}}","push_payment_methods":["crypto"],"processing_time_seconds":6,"min_withdrawal_amount":2}},{"id":"EOS","name":"EOS","min_size":"0.10000000","status":"online","funding_account_id":"992cf114-26e8-4294-95da-9f4123050cae","status_message":"","max_precision":"0.0001000000000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"","network_confirmations":0,"sort_order":45,"crypto_address_link":"https://www.eosx.io/account/{{address}}","crypto_transaction_link":"https://www.eosx.io/tx/{{txId}}","push_payment_methods":["crypto"],"processing_time_seconds":360,"min_withdrawal_amount":1}},{"id":"ZRX","name":"0x","min_size":"0.00001000","status":"online","funding_account_id":"13e1eebd-34db-4125-a464-499dda52d062","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Ξ","network_confirmations":35,"sort_order":90,"crypto_address_link":"https://etherscan.io/token/0xe41d2489571d322189246dafa5ebde1f4699f498?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"BAT","name":"Basic Attention Token","min_size":"1.00000000","status":"online","funding_account_id":"3010021b-c87b-4e59-867e-ea760a528ee4","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Ξ","network_confirmations":35,"sort_order":70,"crypto_address_link":"https://etherscan.io/token/0x0d8775f648430679a709e98d2b0cb6250d2887ef?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"LOOM","name":"Loom Network","min_size":"1.00000000","status":"online","funding_account_id":"8552f2b8-1976-4d61-8a53-76b11d43d899","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Ξ","network_confirmations":35,"sort_order":115,"crypto_address_link":"https://etherscan.io/token/0xa4e8c3ec456107ea67d3075bf9e3df3a75823db0?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"CVC","name":"Civic","min_size":"1.00000000","status":"online","funding_account_id":"1529314a-552f-4ace-9c78-67c2afa85f18","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Ξ","network_confirmations":35,"sort_order":125,"crypto_address_link":"https://etherscan.io/token/0x41e5560054824ea6b0732e656e3ad64e20e94e45?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"DNT","name":"district0x","min_size":"1.00000000","status":"online","funding_account_id":"441a4ab7-b1da-4f69-85dd-cd8533e0f73b","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Ξ","network_confirmations":35,"sort_order":130,"crypto_address_link":"https://etherscan.io/token/0x0abdace70d3790235af448c88547603b945604ea?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"MANA","name":"Decentraland","min_size":"1.00000000","status":"online","funding_account_id":"31b63bed-5023-46d4-81bd-5f27164c049a","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Ξ","network_confirmations":35,"sort_order":110,"crypto_address_link":"https://etherscan.io/token/0x0f5d2fb29fb7d3cfee444a200298f468908cc942?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"GNT","name":"Golem","min_size":"1.00000000","status":"online","funding_account_id":"4f0d150f-c503-4ff1-83eb-42dcb0bd85f5","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Ξ","network_confirmations":35,"sort_order":105,"crypto_address_link":"https://etherscan.io/token/0xa74476443119A942dE498590Fe1f2454d7D4aC0d?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"LINK","name":"Chainlink","min_size":"1.00000000","status":"online","funding_account_id":"c063498d-1863-443a-bca0-c3fec9d84e0b","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Ξ","network_confirmations":35,"sort_order":67,"crypto_address_link":"https://etherscan.io/token/0x514910771af9ca656af840dff83e8264ecf986ca?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"BTC","name":"Bitcoin","min_size":"0.00000001","status":"online","funding_account_id":"db2ca5b7-b734-470a-a1bf-a4c4f1aa4271","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"₿","network_confirmations":3,"sort_order":20,"crypto_address_link":"https://live.blockcypher.com/btc/address/{{address}}","crypto_transaction_link":"https://live.blockcypher.com/btc/tx/{{txId}}","push_payment_methods":["crypto"],"group_types":["btc","crypto"]}},{"id":"EUR","name":"Euro","min_size":"0.01000000","status":"online","funding_account_id":"dfa7a9b4-3c16-4f79-ae1d-8d0292537ded","status_message":"","max_precision":"0.0100000000000000000000000000000000000000","convertible_to":[],"details":{"type":"fiat","symbol":"€","network_confirmations":0,"sort_order":2,"crypto_address_link":"","crypto_transaction_link":"","push_payment_methods":["sepa_bank_account"],"group_types":["fiat","eur"]}},{"id":"LTC","name":"Litecoin","min_size":"0.00000001","status":"online","funding_account_id":"cd64a18e-bacc-49b6-b664-10f1def76481","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Ł","network_confirmations":12,"sort_order":35,"crypto_address_link":"https://live.blockcypher.com/ltc/address/{{address}}","crypto_transaction_link":"https://live.blockcypher.com/ltc/tx/{{txId}}","push_payment_methods":["crypto"]}},{"id":"GBP","name":"British Pound","min_size":"0.01000000","status":"online","funding_account_id":"e0a473f6-364c-42fa-923f-417e8a553c5e","status_message":"","max_precision":"0.0100000000000000000000000000000000000000","convertible_to":[],"details":{"type":"fiat","symbol":"£","network_confirmations":0,"sort_order":3,"crypto_address_link":"","crypto_transaction_link":"","push_payment_methods":["uk_bank_account","swift_lhv","swift"],"group_types":["fiat","gbp"]}},{"id":"USD","name":"United States Dollar","min_size":"0.01000000","status":"online","funding_account_id":"f0bb951e-ef40-4c76-a78a-d80c6bffc887","status_message":"","max_precision":"0.0100000000000000000000000000000000000000","convertible_to":["USDC"],"details":{"type":"fiat","symbol":"$","network_confirmations":0,"sort_order":1,"crypto_address_link":"","crypto_transaction_link":"","push_payment_methods":["bank_wire","fedwire","swift_bank_account","intra_bank_account"],"group_types":["fiat","usd"],"display_name":"US Dollar"}},{"id":"ETH","name":"Ether","min_size":"0.00000001","status":"online","funding_account_id":"3d31bce3-9199-45db-bde4-4a34d3440a1d","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Ξ","network_confirmations":35,"sort_order":25,"crypto_address_link":"https://etherscan.io/address/{{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"],"group_types":["eth","crypto"]}},{"id":"BCH","name":"Bitcoin Cash","min_size":"0.00000001","status":"online","funding_account_id":"aa7b2daf-1521-4685-a5ed-cd2282ab315e","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"₿","network_confirmations":12,"sort_order":40,"crypto_address_link":"https://blockchair.com/bitcoin-cash/address/{{address}}","crypto_transaction_link":"https://blockchair.com/bitcoin-cash/transaction/{{txId}}","push_payment_methods":["crypto"]}},{"id":"ETC","name":"Ether Classic","min_size":"0.00000001","status":"online","funding_account_id":"33db863e-c048-4cbf-93a6-595de85063b4","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"⟠","network_confirmations":80640,"sort_order":55,"crypto_address_link":"https://gastracker.io/addr/{{address}}","crypto_transaction_link":"https://gastracker.io/tx/0x{{txId}}","push_payment_methods":["crypto"]}},{"id":"USDC","name":"USD Coin","min_size":"0.00000100","status":"online","funding_account_id":"0638e043-4136-4365-997b-865607ce915f","status_message":"","max_precision":"0.0000010000000000000000000000000000000000","convertible_to":["USD"],"details":{"type":"crypto","symbol":"$","network_confirmations":35,"sort_order":80,"crypto_address_link":"https://etherscan.io/token/0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"],"group_types":["stablecoin","usdc","crypto"]}},{"id":"ZEC","name":"Zcash","min_size":"0.00000001","status":"online","funding_account_id":"344466b0-22b2-488c-8d52-3c06c5e5de72","status_message":"","max_precision":"0.0000000100000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"ᙇ","network_confirmations":24,"sort_order":65,"crypto_address_link":"https://zcash.blockexplorer.com/address/{{address}}","crypto_transaction_link":"https://zcash.blockexplorer.com/tx/{{txId}}","push_payment_methods":["crypto"]}},{"id":"XTZ","name":"Tezos","min_size":"0.00000100","status":"online","funding_account_id":"2c1f0348-7c10-4153-8f6c-78c0e43a49fc","status_message":"","max_precision":"0.0000010000000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Τ","network_confirmations":60,"sort_order":53,"crypto_address_link":"https://tzstats.com/{{address}}","crypto_transaction_link":"https://tzstats.com/{{txId}}","push_payment_methods":["crypto"],"min_withdrawal_amount":1}},{"id":"DAI","name":"Dai","min_size":"0.00001000","status":"online","funding_account_id":"ac937b93-756d-4541-ae02-e5600d7733c7","status_message":"","max_precision":"0.0000100000000000000000000000000000000000","convertible_to":[],"details":{"type":"crypto","symbol":"Ξ","network_confirmations":35,"sort_order":100,"crypto_address_link":"https://etherscan.io/token/0x89d24a6b4ccb1b6faa2625fe562bdd9a23260359?a={{address}}","crypto_transaction_link":"https://etherscan.io/tx/0x{{txId}}","push_payment_methods":["crypto"],"group_types":["stablecoin","dai","crypto"]}}],"products":[{"id":"LINK-GBP","base_currency":"LINK","quote_currency":"GBP","base_min_size":"1","base_max_size":"90000","base_increment":"0.01","quote_increment":"0.00001","display_name":"LINK/GBP","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"BAND-EUR","base_currency":"BAND","quote_currency":"EUR","base_min_size":"0.1","base_max_size":"18000","base_increment":"0.01","quote_increment":"0.0001","display_name":"BAND/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1.0","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"BAND-GBP","base_currency":"BAND","quote_currency":"GBP","base_min_size":"0.1","base_max_size":"18000","base_increment":"0.01","quote_increment":"0.0001","display_name":"BAND/GBP","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1.0","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"NMR-EUR","base_currency":"NMR","quote_currency":"EUR","base_min_size":"0.01","base_max_size":"3900","base_increment":"0.001","quote_increment":"0.0001","display_name":"NMR/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1.0","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"NMR-GBP","base_currency":"NMR","quote_currency":"GBP","base_min_size":"0.01","base_max_size":"3900","base_increment":"0.001","quote_increment":"0.0001","display_name":"NMR/GBP","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1.0","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"BAND-USD","base_currency":"BAND","quote_currency":"USD","base_min_size":"0.1","base_max_size":"18000","base_increment":"0.01","quote_increment":"0.0001","display_name":"BAND/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1.0","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"BAND-BTC","base_currency":"BAND","quote_currency":"BTC","base_min_size":"0.1","base_max_size":"18000","base_increment":"0.01","quote_increment":"0.00000001","display_name":"BAND/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.0001","max_market_funds":"10","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"NMR-USD","base_currency":"NMR","quote_currency":"USD","base_min_size":"0.01","base_max_size":"3900","base_increment":"0.001","quote_increment":"0.0001","display_name":"NMR/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1.0","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"NMR-BTC","base_currency":"NMR","quote_currency":"BTC","base_min_size":"0.01","base_max_size":"3900","base_increment":"0.001","quote_increment":"0.00000001","display_name":"NMR/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.0001","max_market_funds":"10","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ALGO-GBP","base_currency":"ALGO","quote_currency":"GBP","base_min_size":"1","base_max_size":"500000","base_increment":"1","quote_increment":"0.0001","display_name":"ALGO/GBP","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"XTZ-GBP","base_currency":"XTZ","quote_currency":"GBP","base_min_size":"1","base_max_size":"100000","base_increment":"0.01","quote_increment":"0.00001","display_name":"XTZ/GBP","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"BTC-USD","base_currency":"BTC","quote_currency":"USD","base_min_size":"0.001","base_max_size":"280","base_increment":"0.00000001","quote_increment":"0.01","display_name":"BTC/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"5","max_market_funds":"1000000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"OMG-GBP","base_currency":"OMG","quote_currency":"GBP","base_min_size":"1","base_max_size":"150000","base_increment":"0.1","quote_increment":"0.0001","display_name":"OMG/GBP","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"COMP-USD","base_currency":"COMP","quote_currency":"USD","base_min_size":"0.01","base_max_size":"1700","base_increment":"0.001","quote_increment":"0.01","display_name":"COMP/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1.0","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"DASH-USD","base_currency":"DASH","quote_currency":"USD","base_min_size":"0.01","base_max_size":"1500","base_increment":"0.001","quote_increment":"0.001","display_name":"DASH/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ZRX-USD","base_currency":"ZRX","quote_currency":"USD","base_min_size":"1","base_max_size":"600000","base_increment":"0.00001","quote_increment":"0.000001","display_name":"ZRX/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"REP-USD","base_currency":"REP","quote_currency":"USD","base_min_size":"0.1","base_max_size":"5000","base_increment":"0.000001","quote_increment":"0.01","display_name":"REP/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"30000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ETH-EUR","base_currency":"ETH","quote_currency":"EUR","base_min_size":"0.01","base_max_size":"1600","base_increment":"0.00000001","quote_increment":"0.01","display_name":"ETH/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"400000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"LTC-EUR","base_currency":"LTC","quote_currency":"EUR","base_min_size":"0.1","base_max_size":"1000","base_increment":"0.00000001","quote_increment":"0.01","display_name":"LTC/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"250000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"OMG-EUR","base_currency":"OMG","quote_currency":"EUR","base_min_size":"1","base_max_size":"500000","base_increment":"0.1","quote_increment":"0.0001","display_name":"OMG/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"MKR-USD","base_currency":"MKR","quote_currency":"USD","base_min_size":"0.001","base_max_size":"240","base_increment":"0.000001","quote_increment":"0.0001","display_name":"MKR/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1.0","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"COMP-BTC","base_currency":"COMP","quote_currency":"BTC","base_min_size":"0.01","base_max_size":"1700","base_increment":"0.001","quote_increment":"0.000001","display_name":"COMP/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.0001","max_market_funds":"10","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"LINK-ETH","base_currency":"LINK","quote_currency":"ETH","base_min_size":"1","base_max_size":"90000","base_increment":"0.01","quote_increment":"0.00000001","display_name":"LINK/ETH","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.01","max_market_funds":"400","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"DAI-USDC","base_currency":"DAI","quote_currency":"USDC","base_min_size":"1","base_max_size":"100000","base_increment":"0.00001","quote_increment":"0.000001","display_name":"DAI/USDC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"5","max_market_funds":"100000","post_only":false,"limit_only":true,"cancel_only":false,"type":"spot"},{"id":"LOOM-USDC","base_currency":"LOOM","quote_currency":"USDC","base_min_size":"1","base_max_size":"2500000","base_increment":"1","quote_increment":"0.000001","display_name":"LOOM/USDC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.1","max_market_funds":"100000","post_only":false,"limit_only":true,"cancel_only":false,"type":"spot"},{"id":"DAI-USD","base_currency":"DAI","quote_currency":"USD","base_min_size":"1","base_max_size":"100000","base_increment":"0.00001","quote_increment":"0.000001","display_name":"DAI/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"5","max_market_funds":"100000","post_only":false,"limit_only":true,"cancel_only":false,"type":"spot"},{"id":"CVC-USDC","base_currency":"CVC","quote_currency":"USDC","base_min_size":"1","base_max_size":"2000000","base_increment":"1","quote_increment":"0.000001","display_name":"CVC/USDC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.1","max_market_funds":"100000","post_only":false,"limit_only":true,"cancel_only":false,"type":"spot"},{"id":"DNT-USDC","base_currency":"DNT","quote_currency":"USDC","base_min_size":"1","base_max_size":"10000000","base_increment":"1","quote_increment":"0.000001","display_name":"DNT/USDC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.1","max_market_funds":"100000","post_only":false,"limit_only":true,"cancel_only":false,"type":"spot"},{"id":"GNT-USDC","base_currency":"GNT","quote_currency":"USDC","base_min_size":"1","base_max_size":"1500000","base_increment":"1","quote_increment":"0.000001","display_name":"GNT/USDC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.01","max_market_funds":"100000","post_only":false,"limit_only":true,"cancel_only":false,"type":"spot"},{"id":"ZEC-BTC","base_currency":"ZEC","quote_currency":"BTC","base_min_size":"0.01","base_max_size":"1500","base_increment":"0.0001","quote_increment":"0.000001","display_name":"ZEC/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"30","post_only":false,"limit_only":true,"cancel_only":false,"type":"spot"},{"id":"BAT-ETH","base_currency":"BAT","quote_currency":"ETH","base_min_size":"1","base_max_size":"300000","base_increment":"1","quote_increment":"0.00000001","display_name":"BAT/ETH","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.01","max_market_funds":"500","post_only":false,"limit_only":true,"cancel_only":false,"type":"spot"},{"id":"ETH-DAI","base_currency":"ETH","quote_currency":"DAI","base_min_size":"0.01","base_max_size":"700","base_increment":"0.0001","quote_increment":"0.01","display_name":"ETH/DAI","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":true,"cancel_only":false,"type":"spot"},{"id":"OMG-USD","base_currency":"OMG","quote_currency":"USD","base_min_size":"1","base_max_size":"500000","base_increment":"0.1","quote_increment":"0.0001","display_name":"OMG/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"BTC-EUR","base_currency":"BTC","quote_currency":"EUR","base_min_size":"0.001","base_max_size":"200","base_increment":"0.00000001","quote_increment":"0.01","display_name":"BTC/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"600000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"BTC-GBP","base_currency":"BTC","quote_currency":"GBP","base_min_size":"0.001","base_max_size":"80","base_increment":"0.00000001","quote_increment":"0.01","display_name":"BTC/GBP","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"200000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"BCH-USD","base_currency":"BCH","quote_currency":"USD","base_min_size":"0.01","base_max_size":"700","base_increment":"0.00000001","quote_increment":"0.01","display_name":"BCH/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"500000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"BTC-USDC","base_currency":"BTC","quote_currency":"USDC","base_min_size":"0.001","base_max_size":"280","base_increment":"0.00000001","quote_increment":"0.01","display_name":"BTC/USDC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"1000000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"LTC-USD","base_currency":"LTC","quote_currency":"USD","base_min_size":"0.1","base_max_size":"4000","base_increment":"0.00000001","quote_increment":"0.01","display_name":"LTC/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"250000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ZEC-USDC","base_currency":"ZEC","quote_currency":"USDC","base_min_size":"0.01","base_max_size":"5000","base_increment":"0.00000001","quote_increment":"0.01","display_name":"ZEC/USDC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"250000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"LINK-EUR","base_currency":"LINK","quote_currency":"EUR","base_min_size":"1","base_max_size":"90000","base_increment":"0.01","quote_increment":"0.00001","display_name":"LINK/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ALGO-EUR","base_currency":"ALGO","quote_currency":"EUR","base_min_size":"1","base_max_size":"500000","base_increment":"1","quote_increment":"0.0001","display_name":"ALGO/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"XTZ-EUR","base_currency":"XTZ","quote_currency":"EUR","base_min_size":"1","base_max_size":"100000","base_increment":"0.01","quote_increment":"0.00001","display_name":"XTZ/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ETH-USD","base_currency":"ETH","quote_currency":"USD","base_min_size":"0.01","base_max_size":"2800","base_increment":"0.00000001","quote_increment":"0.01","display_name":"ETH/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"5","max_market_funds":"1000000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ETC-EUR","base_currency":"ETC","quote_currency":"EUR","base_min_size":"0.1","base_max_size":"20000","base_increment":"0.00000001","quote_increment":"0.001","display_name":"ETC/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"KNC-BTC","base_currency":"KNC","quote_currency":"BTC","base_min_size":"1","base_max_size":"600000","base_increment":"0.1","quote_increment":"0.00000001","display_name":"KNC/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"30","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"OXT-USD","base_currency":"OXT","quote_currency":"USD","base_min_size":"1","base_max_size":"500000","base_increment":"1","quote_increment":"0.0001","display_name":"OXT/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ATOM-USD","base_currency":"ATOM","quote_currency":"USD","base_min_size":"0.1","base_max_size":"25000","base_increment":"0.1","quote_increment":"0.001","display_name":"ATOM/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ETH-GBP","base_currency":"ETH","quote_currency":"GBP","base_min_size":"0.01","base_max_size":"1400","base_increment":"0.00000001","quote_increment":"0.01","display_name":"ETH/GBP","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"1000000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"LTC-GBP","base_currency":"LTC","quote_currency":"GBP","base_min_size":"0.1","base_max_size":"1000","base_increment":"0.00000001","quote_increment":"0.01","display_name":"LTC/GBP","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"250000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ATOM-BTC","base_currency":"ATOM","quote_currency":"BTC","base_min_size":"0.1","base_max_size":"25000","base_increment":"0.1","quote_increment":"0.000001","display_name":"ATOM/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"30","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ZRX-BTC","base_currency":"ZRX","quote_currency":"BTC","base_min_size":"1","base_max_size":"600000","base_increment":"0.00001","quote_increment":"0.00000001","display_name":"ZRX/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"60","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"EOS-USD","base_currency":"EOS","quote_currency":"USD","base_min_size":"0.1","base_max_size":"50000","base_increment":"0.1","quote_increment":"0.001","display_name":"EOS/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ZRX-EUR","base_currency":"ZRX","quote_currency":"EUR","base_min_size":"1","base_max_size":"600000","base_increment":"0.00001","quote_increment":"0.000001","display_name":"ZRX/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"XLM-EUR","base_currency":"XLM","quote_currency":"EUR","base_min_size":"1","base_max_size":"600000","base_increment":"1","quote_increment":"0.000001","display_name":"XLM/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.1","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"XLM-USD","base_currency":"XLM","quote_currency":"USD","base_min_size":"1","base_max_size":"600000","base_increment":"1","quote_increment":"0.000001","display_name":"XLM/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.1","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ALGO-USD","base_currency":"ALGO","quote_currency":"USD","base_min_size":"1","base_max_size":"500000","base_increment":"1","quote_increment":"0.0001","display_name":"ALGO/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"XTZ-USD","base_currency":"XTZ","quote_currency":"USD","base_min_size":"1","base_max_size":"100000","base_increment":"0.01","quote_increment":"0.0001","display_name":"XTZ/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ETC-GBP","base_currency":"ETC","quote_currency":"GBP","base_min_size":"0.1","base_max_size":"20000","base_increment":"0.00000001","quote_increment":"0.001","display_name":"ETC/GBP","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ETC-USD","base_currency":"ETC","quote_currency":"USD","base_min_size":"0.1","base_max_size":"20000","base_increment":"0.00000001","quote_increment":"0.001","display_name":"ETC/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"XRP-BTC","base_currency":"XRP","quote_currency":"BTC","base_min_size":"1","base_max_size":"500000","base_increment":"1","quote_increment":"0.00000001","display_name":"XRP/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"30","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"OMG-BTC","base_currency":"OMG","quote_currency":"BTC","base_min_size":"1","base_max_size":"150000","base_increment":"0.1","quote_increment":"0.00000001","display_name":"OMG/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"500","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"MKR-BTC","base_currency":"MKR","quote_currency":"BTC","base_min_size":"0.001","base_max_size":"240","base_increment":"0.000001","quote_increment":"0.00001","display_name":"MKR/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.0001","max_market_funds":"11","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"KNC-USD","base_currency":"KNC","quote_currency":"USD","base_min_size":"1","base_max_size":"500000","base_increment":"0.1","quote_increment":"0.0001","display_name":"KNC/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"1","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ETH-BTC","base_currency":"ETH","quote_currency":"BTC","base_min_size":"0.01","base_max_size":"2400","base_increment":"0.00000001","quote_increment":"0.00001","display_name":"ETH/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"80","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"LTC-BTC","base_currency":"LTC","quote_currency":"BTC","base_min_size":"0.1","base_max_size":"8000","base_increment":"0.00000001","quote_increment":"0.000001","display_name":"LTC/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"120","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"BCH-GBP","base_currency":"BCH","quote_currency":"GBP","base_min_size":"0.01","base_max_size":"250","base_increment":"0.00000001","quote_increment":"0.01","display_name":"BCH/GBP","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"500000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"BCH-BTC","base_currency":"BCH","quote_currency":"BTC","base_min_size":"0.01","base_max_size":"400","base_increment":"0.00000001","quote_increment":"0.00001","display_name":"BCH/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"60","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ETC-BTC","base_currency":"ETC","quote_currency":"BTC","base_min_size":"0.1","base_max_size":"5000","base_increment":"0.00000001","quote_increment":"0.000001","display_name":"ETC/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"30","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"BAT-USDC","base_currency":"BAT","quote_currency":"USDC","base_min_size":"1","base_max_size":"800000","base_increment":"1","quote_increment":"0.000001","display_name":"BAT/USDC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"ETH-USDC","base_currency":"ETH","quote_currency":"USDC","base_min_size":"0.01","base_max_size":"2800","base_increment":"0.00000001","quote_increment":"0.01","display_name":"ETH/USDC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"1000000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"EOS-BTC","base_currency":"EOS","quote_currency":"BTC","base_min_size":"0.1","base_max_size":"50000","base_increment":"0.1","quote_increment":"0.000001","display_name":"EOS/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"30","post_only":false,"limit_only":true,"cancel_only":false,"type":"spot"},{"id":"REP-BTC","base_currency":"REP","quote_currency":"BTC","base_min_size":"0.1","base_max_size":"5000","base_increment":"0.000001","quote_increment":"0.000001","display_name":"REP/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"6","post_only":false,"limit_only":true,"cancel_only":false,"type":"spot"},{"id":"MANA-USDC","base_currency":"MANA","quote_currency":"USDC","base_min_size":"1","base_max_size":"2800000","base_increment":"1","quote_increment":"0.000001","display_name":"MANA/USDC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.1","max_market_funds":"100000","post_only":false,"limit_only":true,"cancel_only":false,"type":"spot"},{"id":"LINK-USD","base_currency":"LINK","quote_currency":"USD","base_min_size":"1","base_max_size":"90000","base_increment":"0.01","quote_increment":"0.00001","display_name":"LINK/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"EOS-EUR","base_currency":"EOS","quote_currency":"EUR","base_min_size":"0.1","base_max_size":"50000","base_increment":"0.1","quote_increment":"0.001","display_name":"EOS/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"XTZ-BTC","base_currency":"XTZ","quote_currency":"BTC","base_min_size":"1","base_max_size":"100000","base_increment":"0.01","quote_increment":"0.00000001","display_name":"XTZ/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"10","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"XRP-GBP","base_currency":"XRP","quote_currency":"GBP","base_min_size":"1","base_max_size":"500000","base_increment":"0.000001","quote_increment":"0.0001","display_name":"XRP/GBP","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"XLM-BTC","base_currency":"XLM","quote_currency":"BTC","base_min_size":"1","base_max_size":"600000","base_increment":"1","quote_increment":"0.00000001","display_name":"XLM/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.001","max_market_funds":"50","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"DASH-BTC","base_currency":"DASH","quote_currency":"BTC","base_min_size":"0.01","base_max_size":"1500","base_increment":"0.001","quote_increment":"0.00000001","display_name":"DASH/BTC","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"0.0001","max_market_funds":"10","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"XRP-EUR","base_currency":"XRP","quote_currency":"EUR","base_min_size":"1","base_max_size":"500000","base_increment":"0.000001","quote_increment":"0.0001","display_name":"XRP/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"XRP-USD","base_currency":"XRP","quote_currency":"USD","base_min_size":"1","base_max_size":"500000","base_increment":"0.000001","quote_increment":"0.0001","display_name":"XRP/USD","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"100000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"},{"id":"BCH-EUR","base_currency":"BCH","quote_currency":"EUR","base_min_size":"0.01","base_max_size":"100","base_increment":"0.00000001","quote_increment":"0.01","display_name":"BCH/EUR","status":"online","margin_enabled":false,"status_message":"","min_market_funds":"10","max_market_funds":"300000","post_only":false,"limit_only":false,"cancel_only":false,"type":"spot"}]}
    "#;
    match serde_json::from_str(msg)? {
      ResponseMessages::Status { resp: _ } => {},
      _ => {
        assert!(false)
      }
    }
    return Ok(());
  }

  #[test]
  fn test_match() -> Result<(), serde_json::error::Error> {
    let msg = r#"
    {
    "type":"match",
    "trade_id":62995921,
    "maker_order_id":"125f1d3d-3100-41ce-9341-fc330bdcebcb",
    "taker_order_id":"5b0a9f2d-3388-4fd4-a106-b96b1e6d302f",
    "side":"buy",
    "size":"1.9",
    "price":"434.19",
    "product_id":"ETH-USD",
    "sequence":10182385681,
    "time":"2020-08-31T15:05:14.336755Z"
    }
    "#;
    match serde_json::from_str(msg)? {
      ResponseMessages::Match { resp: _ } => {},
      _ => {
        assert!(false);
      }
    }
    return Ok(());
  }

  #[test]
  fn test_done() -> Result<(), serde_json::error::Error> {
    let msg = r#"
    {
    "type":"done",
    "side":"buy",
    "product_id":"ETH-EUR",
    "time":"2020-08-31T15:15:01.044966Z",
    "sequence":4744937283,
    "order_id":"64093d3f-1d99-4858-a5af-1c85e0c83e48",
    "reason":"filled"
    }
    "#;
    match serde_json::from_str(msg)? {
      ResponseMessages::Done { resp: _ } => {},
      _ => { assert!(false) }
    };
    return Ok(());
  }
}