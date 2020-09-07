use std::collections::HashSet;
use std::iter::FromIterator;
use std::sync::Mutex; // TODO maybe replace this with parking_log::Mutex if necessary.
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam::{Sender, TryRecvError, Receiver};
use log;
use tungstenite::{Message, WebSocket};
use tungstenite::client::AutoStream;
use url::Url;

use super::common::Channel;
use super::CoinBaseWebSocketMessageHandler;
use super::request::{SubscribeRequest, UnsubscribeRequest};
use super::RequestMessages;
use super::response;


enum WebSocketWorkerMessages {
  Subscribe { product_ids: Vec<String>, channels: Vec<Channel> },
  Unsubscribe { product_ids: Vec<String>, channels: Vec<Channel> },
  Stop,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum ClientState {
  NotInitialized,
  Running,
  Stopped,
}


pub struct CoinbaseWebSocketClient {
  url: String,

  state: ClientState,
  lock: Mutex<()>,
  sender: Sender<WebSocketWorkerMessages>,
  receiver: Receiver<WebSocketWorkerMessages>,
  join_handle: Option<JoinHandle<()>>,
}

impl CoinbaseWebSocketClient {
  fn new(url: &str) -> Self {
    let (sender, receiver) = crossbeam::bounded(10);
    CoinbaseWebSocketClient {
      url: url.into(),
      state: ClientState::NotInitialized,
      lock: Mutex::new(()),
      sender, receiver,
      join_handle: None,
    }
  }

  pub fn production() -> Self {
    CoinbaseWebSocketClient::new("wss://ws-feed.pro.coinbase.com")
  }

  pub fn sandbox() -> Self {
    CoinbaseWebSocketClient::new("wss://ws-feed-public.sandbox.pro.coinbase.com")
  }

  pub fn start<T: CoinBaseWebSocketMessageHandler + Send + 'static>(&mut self, handler: T) {
    let _guard = self.lock.lock().unwrap();
    if self.state != ClientState::NotInitialized {
      panic!("Client is in state {:?}", self.state); // TODO add appropriate error.
    }

    let receiver = self.receiver.clone();
    let url = self.url.clone();
    let join_handle = thread::spawn(move || {
      let mut worker = CoinBaseWebSocketClientWorker {
        url: Url::parse(url.as_str()).unwrap(),
        last_connect_time: None,
        receiver,
        opt_socket: None,
        product_ids: HashSet::new(),
        channels: HashSet::new(),
        handler,
      };
      worker.run();
    });
    self.join_handle = Some(join_handle);
    self.state = ClientState::Running
  }

  pub fn controller(&self) -> CoinbaseWebSocketClientController {
    CoinbaseWebSocketClientController {
      sender: self.sender.clone()
    }
  }

  pub fn stop(mut self) {
    let _guard = self.lock.lock().unwrap();
    match self.state {
      ClientState::NotInitialized => {
        log::info!("Client stop was called but client was not started.");
        return;
      },
      ClientState::Stopped => {
        log::info!("Client stopped multiple times");
        return;
      },
      _ => { /* ignore */ }
    }
    // Note: Sender must be set otherwise it is an error and it should panic.
    match self.sender.send(WebSocketWorkerMessages::Stop) {
      Err(_) => {
        log::error!("Couldn't send stop message to the worker since channel was closed.");
      },
      _ => { /* ignore */ }
    };
    self.join_handle.take().unwrap()
      .join()
      .expect("Got error while joining worker thread.");
    self.state = ClientState::Stopped;
  }

  pub fn wait(mut self) {
    let _guard = self.lock.lock().unwrap();
    match self.state {
      ClientState::NotInitialized => {
        log::info!("Client stop was called but client was not started.");
        return;
      },
      ClientState::Stopped => {
        log::info!("Client stopped multiple times");
        return;
      },
      _ => { /* ignore */ }
    }
    let join_handle = self.join_handle.take().unwrap();
    self.state = ClientState::Stopped;
    drop(_guard);
    let _ = join_handle.join();
  }
}

pub struct CoinbaseWebSocketClientController {
  sender: Sender<WebSocketWorkerMessages>,
}

impl CoinbaseWebSocketClientController {

  pub fn subscribe(
    &self,
    product_ids: Vec<String>,
    channels: Vec<Channel>,
  ) {
    self.send_message(WebSocketWorkerMessages::Subscribe { product_ids, channels });
  }

  pub fn unsubscribe(
    &self,
    product_ids: Vec<String>,
    channels: Vec<Channel>,
  ) {
    self.send_message(WebSocketWorkerMessages::Unsubscribe { product_ids, channels });
  }

  fn send_message(&self, message: WebSocketWorkerMessages) {
    match self.sender.send(message) {
      Err(_) => {
        log::warn!("Got error while sending message to the websocket worker because channel is closed.");
      },
      _ => { /* ignore */ }
    };
  }
}

const WEBSOCKET_WORKER_ID: &str = "WebSocketWorker";

enum TerminateOrReconnect {
  Reconnect,
  Terminal,
}

struct CoinBaseWebSocketClientWorker<T: CoinBaseWebSocketMessageHandler> {
  url: Url,
  last_connect_time: Option<Instant>,
  receiver: crossbeam::Receiver<WebSocketWorkerMessages>,
  opt_socket: Option<WebSocket<AutoStream>>,
  product_ids: HashSet<String>,
  channels: HashSet<Channel>,
  handler: T,
}

impl<T: CoinBaseWebSocketMessageHandler> CoinBaseWebSocketClientWorker<T> {
  fn run(&mut self) {
    if self.wait_until_initial_connection().is_err() {
      // Note technically this can be both terminal and reconnect errors,
      // since subscribe can return reconnect error, but if we were not
      // able to establish initial connection and subscription then we
      // opt out from trying to establish any further connections.
      log::warn!(target: WEBSOCKET_WORKER_ID, "Initial connection could not be established.");
      return;
    }

    log::trace!("Initial connection acquired");
    match self.handler.initialize() {
      Err(_) => {
        log::warn!("Got terminate signal from the handler.");
        return;
      },
      _ => { /* ignore */ }
    }
    log::trace!("Initializing handler");

    // Main event loop.
    loop {
      let res = self.step();
      if res.is_err() {
        match res.unwrap_err() {
          TerminateOrReconnect::Reconnect => {
            if self.connect().and_then(|_| self.subscribe()).is_err() {
              log::warn!(target: WEBSOCKET_WORKER_ID, "Could not reconnect to the web socket stream.");
              return;
            }
          }
          TerminateOrReconnect::Terminal => return
        };
      }
    }
  }


  fn step(&mut self) -> Result<(), TerminateOrReconnect> {
    match self.receiver.try_recv() {
      Ok(msg) => {
        match msg {
          WebSocketWorkerMessages::Subscribe { product_ids, channels } => {
            // Subscribe to new channels.
            log::debug!(target: WEBSOCKET_WORKER_ID, "Got subscribe message for products: {:?}, and channels: {:?}", product_ids, channels);
            self.append_subscriptions(&product_ids, &channels);
            self.subscribe()
          }
          WebSocketWorkerMessages::Unsubscribe { product_ids, channels } => {
            // Unsubscribe from some channels.
            log::debug!(target: WEBSOCKET_WORKER_ID, "Got unsubscribe message for products: {:?}, and channels: {:?}", product_ids, channels);
            self.remove_subscriptions(&product_ids, &channels);
            self.unsubscribe(product_ids, channels)
          }
          WebSocketWorkerMessages::Stop => {
            // Exit gracefully.
            log::info!(target: WEBSOCKET_WORKER_ID, "Got stop signal for web socket stream");
            return Err(TerminateOrReconnect::Terminal);
          }
        }
      }
      Err(error) => {
        match error {
          TryRecvError::Empty => self.consume_socket(),
          TryRecvError::Disconnected => {
            // Exit with error.
            log::error!(target: WEBSOCKET_WORKER_ID, "Message Channel closed from outside. This is illegal state.");
            Err(TerminateOrReconnect::Terminal)
          }
        }
      }
    }
  }

  fn connect(&mut self) -> Result<(), TerminateOrReconnect> {
    loop {
      let can_try_to_connect = self.last_connect_time
        .map(|instant| instant + Duration::from_millis(500) < Instant::now())
        .unwrap_or(true);

      if can_try_to_connect {
        match tungstenite::connect(&self.url) {
          Ok((socket, http_response)) => {
            log::info!(target: WEBSOCKET_WORKER_ID, "Connected to the server");
            log::info!(target: WEBSOCKET_WORKER_ID, "Response HTTP code: {}", http_response.status());
            log::info!(target: WEBSOCKET_WORKER_ID, "Response contains the following headers:");
            for (header, value) in http_response.headers() {
              log::info!(target: WEBSOCKET_WORKER_ID, "{}: {:?}", header, value);
            }
            self.opt_socket = Some(socket); // Last socket will be dropped here.
            return Ok(());
          }
          Err(error) => {
            match error {
              tungstenite::Error::ConnectionClosed => {
                // Connection was closed normally, meaning that
                // this is an illegal state and it is ok to terminate here.
                log::warn!(target: WEBSOCKET_WORKER_ID, "Trying to work with web-socket after connection was manually closed.");
                return Err(TerminateOrReconnect::Terminal);
              }
              error => {
                // Just log errors and ignore.
                log::warn!(target: WEBSOCKET_WORKER_ID, "Got error while connecting: {:?}", error);
              }
            }
            self.last_connect_time = Some(Instant::now());
          }
        };
      } else {
        log::debug!(target: WEBSOCKET_WORKER_ID, "Going to sleep before reconnect for 250 millis");
        thread::sleep(Duration::from_millis(250))
      }
    }
  }

  fn append_subscriptions(&mut self, product_ids: &Vec<String>, channels: &Vec<Channel>) {
    self.product_ids.extend(product_ids.clone());
    self.channels.extend(channels.clone());
  }

  fn subscribe(&mut self) -> Result<(), TerminateOrReconnect> {
    let product_ids = Vec::from_iter(self.product_ids.clone());
    let channels = Vec::from_iter(self.channels.clone());
    self.send_request(
      RequestMessages::Subscribe { req: SubscribeRequest::new(product_ids, channels) }
    )
  }

  fn remove_subscriptions(&mut self, product_ids: &Vec<String>, channels: &Vec<Channel>) {
    product_ids.iter().for_each(|product| { self.product_ids.remove(product); });
    channels.iter().for_each(|product| { self.channels.remove(product); });
  }

  fn unsubscribe(&mut self, product_ids: Vec<String>, channels: Vec<Channel>) -> Result<(), TerminateOrReconnect> {
    self.send_request(
      RequestMessages::Unsubscribe { req: UnsubscribeRequest::new(product_ids, channels) }
    )
  }

  fn send_request(&mut self, request: RequestMessages) -> Result<(), TerminateOrReconnect> {
    let socket = self.opt_socket.as_mut().unwrap();
    let json_msg = serde_json::to_string(&request).unwrap();
    socket.write_message(Message::text(json_msg)).or_else(|err| {
      log::debug!(target: WEBSOCKET_WORKER_ID, "Got error while sending subscribe message ");
      handle_ws_error(err)
    })
  }

  fn wait_until_initial_connection(&mut self) -> Result<(), TerminateOrReconnect> {
    let started = Instant::now();
    loop {
      match self.receiver.try_recv() {
        Ok(msg) => {
          match msg {
            WebSocketWorkerMessages::Subscribe { product_ids, channels } => {
              log::info!("Got subscribe message: product_ids: {:?} | channels: {:?}", &product_ids, &channels);
              self.append_subscriptions(&product_ids, &channels);
              return self.connect()
                .and_then(|_| self.subscribe());
            }
            WebSocketWorkerMessages::Unsubscribe { product_ids: _, channels: _ } => {
              log::warn!(target: WEBSOCKET_WORKER_ID, "Got unsubscribe message, but no initial connection was establish.");
              continue;
            }
            WebSocketWorkerMessages::Stop => {
              log::warn!("Got stop message before initial connection was established");
              // Exit but gracefully
              return Ok(());
            }
          }
        }
        Err(err) => {
          match err {
            TryRecvError::Empty => {
              // Just wait
              if started.elapsed() > Duration::from_secs(15) {
                log::warn!(target: WEBSOCKET_WORKER_ID, "Haven't subscribed for {} seconds.", started.elapsed().as_secs());
              }
              log::debug!(target: WEBSOCKET_WORKER_ID, "Sleeping 1 second");
              thread::sleep(Duration::from_secs(1));
            }
            TryRecvError::Disconnected => {
              log::error!(target: WEBSOCKET_WORKER_ID, "Communication channel closed. This is illegal ");
              return Err(TerminateOrReconnect::Terminal);
            }
          }
        }
      }
    }
  }

  fn consume_socket(&mut self) -> Result<(), TerminateOrReconnect> {
    // UNWRAP if we ever get here and socket is empty then this should panic
    // because that is an illegal state.
    let socket = self.opt_socket.as_mut().unwrap();
    match socket.read_message() {
      Ok(msg) => self.handle_ws_message(msg),
      Err(err) => {
        log::warn!(target: WEBSOCKET_WORKER_ID, "Got web socket error while consuming web socket message");
        handle_ws_error(err)
      }
    }
  }

  fn handle_ws_message(&mut self, message: Message) -> Result<(), TerminateOrReconnect> {
    match message {
      Message::Text(json) => {
        log::debug!(target: WEBSOCKET_WORKER_ID, "{}", json);
        return self.handle_message(json);
      }
      Message::Close(opt_close_frame) => {
        log::info!(target: WEBSOCKET_WORKER_ID, "Got WebSocket::Close message from stream.");
        if opt_close_frame.is_some() {
          let close_frame = opt_close_frame.unwrap();
          log::info!(target: WEBSOCKET_WORKER_ID, "Close code: {} | Close reason: {}", close_frame.code, close_frame.reason);
        }
        return Err(TerminateOrReconnect::Reconnect);
      }
      Message::Ping(_) => log::debug!(target: WEBSOCKET_WORKER_ID, "WebSocket::Ping"),
      Message::Pong(_) => log::debug!(target: WEBSOCKET_WORKER_ID, "WebSocket::Pong"),
      Message::Binary(_) => log::warn!(target: WEBSOCKET_WORKER_ID, "WebSocket binary message received?")
    };
    Ok(())
  }

  fn handle_message(&mut self, json_msg: String) -> Result<(), TerminateOrReconnect> {
    let response_result = serde_json::from_str(json_msg.as_str());
    if response_result.is_err() {
      log::warn!(target: WEBSOCKET_WORKER_ID, "Could not parse following message from the coinbase: \n {}", json_msg);
      return Ok(()); // Just ignore the message.
    }

    // @formatter:off
    match response_result.unwrap() {
      response::ResponseMessages::Subscriptions { resp } => { self.handler.on_subscriptions(&resp) }
      response::ResponseMessages::Heartbeat     { resp } => { self.handler.on_heartbeat(&resp)     }
      response::ResponseMessages::Status        { resp } => { self.handler.on_status(&resp)        }
      response::ResponseMessages::Ticker        { resp } => { self.handler.on_ticker(&resp)        }
      response::ResponseMessages::Snapshot      { resp } => { self.handler.on_snapshot(&resp)      }
      response::ResponseMessages::L2Update      { resp } => { self.handler.on_l2_update(&resp)     }
      response::ResponseMessages::Match         { resp } => { self.handler.on_match(&resp)         }
      response::ResponseMessages::Received      { resp } => { self.handler.on_received(&resp)      }
      response::ResponseMessages::Open          { resp } => { self.handler.on_open(&resp)          }
      response::ResponseMessages::Change        { resp } => { self.handler.on_change(&resp)        }
      response::ResponseMessages::Done          { resp } => { self.handler.on_done(&resp)          }
      response::ResponseMessages::Active        { resp } => { self.handler.on_active(&resp)        }
      response::ResponseMessages::Last_Match    { resp } => { self.handler.on_last_match(&resp)    }
      response::ResponseMessages::Error         { resp } => { self.handler.on_error(&resp)         }
    }.map_err(|_| TerminateOrReconnect::Terminal)
    // @formatter:on
  }
}

fn handle_ws_error(error: tungstenite::Error) -> Result<(), TerminateOrReconnect> {
  match error {
    tungstenite::Error::ConnectionClosed => {
      // Connection was closed normally, meaning that
      // this is an illegal state and it is ok to panic here
      log::warn!(target: WEBSOCKET_WORKER_ID, "Trying to work with web-socket after connection was manually closed.");
      Err(TerminateOrReconnect::Terminal)
    }
    tungstenite::Error::AlreadyClosed => {
      // Connection was closed for some reason and we need to try to reconnect.
      log::warn!(target: WEBSOCKET_WORKER_ID, "WebSocket connection is closed for unknown reason.");
      Err(TerminateOrReconnect::Reconnect)
    }
    tungstenite::Error::Io(error) => {
      // Once io error happen we can choose to either reconnect or try to reconnect.
      // Here we choose to just drop current connection and try to reconnect since
      // IO errors are produced for a lot of different reasons some of which might pass
      // after some time (e.g. timeout due to large load or missing network connection).
      log::warn!(target: WEBSOCKET_WORKER_ID, "Got an IO error, {:?}. ", error);
      Err(TerminateOrReconnect::Reconnect)
    }
    error => {
      // Just log errors and ignore.
      log::warn!(target: WEBSOCKET_WORKER_ID, "Got error: {:?}", error);
      Ok(())
    }
  }
}