use serde::{Deserialize, Serialize};

use super::common::Channel;

// @formatter:off
#[serde(tag = "type", rename_all = "lowercase")]
#[derive(Serialize, Deserialize, Debug)]
pub enum RequestMessages {
  Subscribe   { #[serde(flatten)] req: SubscribeRequest },
  Unsubscribe { #[serde(flatten)] req: UnsubscribeRequest },
}
// @formatter:on


#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeRequest {
  pub product_ids: Vec<String>,
  pub channels: Vec<Channel>,
}

impl SubscribeRequest {
  pub fn new(product_ids: Vec<String>, channels: Vec<Channel>) -> Self {
    SubscribeRequest { product_ids, channels }
  }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct UnsubscribeRequest {
  product_ids: Option<Vec<String>>,
  channels: Vec<Channel>,
}

impl UnsubscribeRequest {
  pub fn unsubscribe_from_channels(channels: Vec<Channel>) -> Self {
    UnsubscribeRequest { product_ids: None, channels }
  }

  pub fn new(product_ids: Vec<String>, channels: Vec<Channel>) -> Self {
    UnsubscribeRequest { product_ids: Some(product_ids), channels }
  }
}