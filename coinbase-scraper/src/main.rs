use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, LineWriter, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;

use coinbase::web_socket::common::{Channel, Channels};
use coinbase::web_socket::response;
use coinbase::web_socket::{CoinbaseWebSocketClient};
use coinbase::web_socket::{CoinBaseWebSocketMessageHandler, Terminate};


struct WriteToFileVisitor {
  writers: HashMap<String, LineWriter<File>>,
  directory: PathBuf,
}

impl WriteToFileVisitor {

  fn new(directory: PathBuf) -> Self {
    return WriteToFileVisitor { directory, writers: HashMap::new() };
  }

  fn write<T: Serialize>(&mut self, value: T, id: String) {
    let mut file_path = self.directory.clone();
    file_path.push(&id);
    let writer = self.writers.entry(id).or_insert_with(move || {
      let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(&file_path)
        .expect(format!("Could not open file {}", &file_path.to_string_lossy()).as_str());
      LineWriter::new(file)
    });

    let line = serde_json::to_string(&value).unwrap();
    writer.write_all(line.as_bytes()).expect("");
    writer.write_all(b"\n").expect("");
  }
}

impl Drop for WriteToFileVisitor {
  fn drop(&mut self) {
    for writer in self.writers.values_mut() {
      writer.flush();
    }
  }
}

impl CoinBaseWebSocketMessageHandler for WriteToFileVisitor {
  fn on_ticker(&mut self, resp: &response::TickerResponse) -> Result<(), Terminate> {
    let mut id = "ticker_".to_string();
    id.push_str(resp.product_id.as_str());
    self.write(resp, id);
    Ok(())
  }

  fn on_l2_update(&mut self, resp: &response::L2UpdateResponse) -> Result<(), Terminate> {
    let mut id = "l2update_".to_string();
    id.push_str(resp.product_id.as_str());
    self.write(resp, id);
    Ok(())
  }
}

fn main() -> anyhow::Result<()> {
  env_logger::init();

  let directory = std::env::args().nth(1).unwrap();
  let mut client = CoinbaseWebSocketClient::production();
  let visitor = WriteToFileVisitor::new(
    PathBuf::from(directory)
  );


  let product_ids = vec![
    "GNT-USDC",
    "BAT-ETH",
    "XRP-EUR",
    "BCH-GBP",
    "XTZ-USD",
    "XLM-EUR",
    "ETC-BTC",
    "ETC-GBP",
    "EOS-USD",
    "LINK-USD",
    "LTC-GBP",
    "BAND-BTC",
    "MKR-BTC",
    "ETH-BTC",
    "KNC-USD",
    "XTZ-BTC",
    "NMR-EUR",
    "BAT-USDC",
    "ALGO-USD",
    "LTC-USD",
    "BCH-USD",
    "CGLD-USD",
    "OXT-USD",
    "ATOM-BTC",
    "DAI-USDC",
    "ETH-EUR",
    "REP-BTC",
    "XLM-USD",
    "EOS-BTC",
    "ZEC-BTC",
    "ATOM-USD",
    "OMG-EUR",
    "LTC-BTC",
    "ETH-USD",
    "DAI-USD",
    "XTZ-GBP",
    "LTC-EUR",
    "BTC-USD",
    "LINK-GBP",
    "XLM-BTC",
    "ALGO-GBP",
    "ZRX-USD",
    "OMG-USD",
    "ETH-USDC",
    "DNT-USDC",
    "BAND-EUR",
    "BTC-USDC",
    "DASH-BTC",
    "ALGO-EUR",
    "LINK-EUR",
    "DASH-USD",
    "XRP-GBP",
    "COMP-USD",
    "XRP-USD",
    "BTC-EUR",
    "EOS-EUR",
    "CGLD-GBP",
    "XTZ-EUR",
    "NMR-BTC",
    "ZEC-USDC",
    "ETC-USD",
    "REP-USD",
    "BAND-USD",
    "BCH-BTC",
    "KNC-BTC",
    "MKR-USD",
    "ETH-GBP",
    "XRP-BTC",
    "OMG-GBP",
    "OMG-BTC",
    "ETC-EUR",
    "CVC-USDC",
    "ZRX-EUR",
    "CGLD-BTC",
    "BTC-GBP",
    "COMP-BTC",
    "ZRX-BTC",
    "BCH-EUR",
    "ETH-DAI",
    "LINK-ETH",
    "NMR-USD",
    "BAND-GBP",
    "NMR-GBP",
    "CGLD-EUR",
    "MANA-USDC",
    "LOOM-USDC",
  ];
  let product_ids = product_ids.into_iter().map(|name| name.to_string()).collect();

  client.start(visitor);
  let controller = client.controller();
  controller.subscribe(
    product_ids, Channel::from_names(&[Channels::Ticker]),
  );
  client.wait();

  return Ok(());
}