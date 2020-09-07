use super::response;

#[derive(Debug)]
pub struct Terminate;

// @formatter:off
pub trait CoinBaseWebSocketMessageHandler {
  fn initialize      (&mut self                                        ) -> Result<(), Terminate> { Ok(()) }
  fn on_subscriptions(&mut self, _resp: &response::SubscriptionResponse) -> Result<(), Terminate> { Ok(()) }
  fn on_heartbeat    (&mut self, _resp: &response::HeartBeatResponse   ) -> Result<(), Terminate> { Ok(()) }
  fn on_status       (&mut self, _resp: &response::StatusResponse      ) -> Result<(), Terminate> { Ok(()) }
  fn on_ticker       (&mut self, _resp: &response::TickerResponse      ) -> Result<(), Terminate> { Ok(()) }
  fn on_snapshot     (&mut self, _resp: &response::SnapshotResponse    ) -> Result<(), Terminate> { Ok(()) }
  fn on_l2_update    (&mut self, _resp: &response::L2UpdateResponse    ) -> Result<(), Terminate> { Ok(()) }
  fn on_match        (&mut self, _resp: &response::MatchResponse       ) -> Result<(), Terminate> { Ok(()) }
  fn on_received     (&mut self, _resp: &response::ReceivedResponse    ) -> Result<(), Terminate> { Ok(()) }
  fn on_open         (&mut self, _resp: &response::OpenResponse        ) -> Result<(), Terminate> { Ok(()) }
  fn on_change       (&mut self, _resp: &response::ChangeResponse      ) -> Result<(), Terminate> { Ok(()) }
  fn on_done         (&mut self, _resp: &response::DoneResponse        ) -> Result<(), Terminate> { Ok(()) }
  fn on_active       (&mut self, _resp: &response::ActiveResponse      ) -> Result<(), Terminate> { Ok(()) }
  fn on_last_match   (&mut self, _resp: &response::LastMatchResponse   ) -> Result<(), Terminate> { Ok(()) }
  fn on_error        (&mut self, _resp: &response::ErrorResponse       ) -> Result<(), Terminate> { Ok(()) }
  fn close           (&mut self                                        ) -> Result<(), Terminate> { Ok(()) }
}
// @formatter:on


pub struct CompositeCoinBaseWebSocketMessageHandler {
  handlers: Vec<Box<dyn CoinBaseWebSocketMessageHandler>>
}

impl CompositeCoinBaseWebSocketMessageHandler {
  pub fn new(handlers: Vec<Box<dyn CoinBaseWebSocketMessageHandler>>) -> Self {
    CompositeCoinBaseWebSocketMessageHandler { handlers }
  }
}

macro_rules! compose_visitors {
  ($self:expr, $fn:ident $(,$opt_argument:expr)?) => {{
    use std::vec::Vec;
    let mut errors = Vec::with_capacity(0);
    for handler in $self.handlers.iter_mut() {
      match handler.$fn($($opt_argument)?) {
        Err(error) => errors.push(error),
        _ => {}
      }
    }

    if !errors.is_empty() {
      return Err(Terminate);
    }

    Ok(())
  }}
}

impl CoinBaseWebSocketMessageHandler for CompositeCoinBaseWebSocketMessageHandler {

  fn initialize(&mut self) -> Result<(), Terminate> {
    compose_visitors!(self, initialize)
  }

  fn on_subscriptions(&mut self, resp: &response::SubscriptionResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_subscriptions, resp)
  }

  fn on_heartbeat(&mut self, resp: &response::HeartBeatResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_heartbeat, resp)
  }

  fn on_status(&mut self, resp: &response::StatusResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_status, resp)
  }

  fn on_ticker(&mut self, resp: &response::TickerResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_ticker, resp)
  }

  fn on_snapshot(&mut self, resp: &response::SnapshotResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_snapshot, resp)
  }

  fn on_l2_update(&mut self, resp: &response::L2UpdateResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_l2_update, resp)
  }

  fn on_match(&mut self, resp: &response::MatchResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_match, resp)
  }

  fn on_received(&mut self, resp: &response::ReceivedResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_received, resp)
  }

  fn on_open(&mut self, resp: &response::OpenResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_open, resp)
  }

  fn on_change(&mut self, resp: &response::ChangeResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_change, resp)
  }

  fn on_done(&mut self, resp: &response::DoneResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_done, resp)
  }

  fn on_active(&mut self, resp: &response::ActiveResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_active, resp)
  }

  fn on_last_match(&mut self, resp: &response::LastMatchResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_last_match, resp)
  }

  fn on_error(&mut self, resp: &response::ErrorResponse) -> Result<(), Terminate> {
    compose_visitors!(self, on_error, resp)
  }

  fn close(&mut self) -> Result<(), Terminate> {
    compose_visitors!(self, close)
  } // Return None by default.
}