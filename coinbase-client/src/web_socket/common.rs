use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, MapAccess, Visitor};
use serde::export::Formatter;
use serde::ser::SerializeStruct;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Channels {
  Heartbeat,
  Status,
  Ticker,
  Level2,
  Matches,
  User,
  Full,
}

impl FromStr for Channels {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "heartbeat" => Ok(Channels::Heartbeat),
      "status" => Ok(Channels::Status),
      "ticker" => Ok(Channels::Ticker),
      "level2" => Ok(Channels::Level2),
      "matches" => Ok(Channels::Matches),
      "user" => Ok(Channels::User),
      "full" => Ok(Channels::Full),
      _ => Err(())
    }
  }
}

impl ToString for Channels {

  fn to_string(&self) -> String {
    match self {
      Channels::Heartbeat => "heartbeat".to_string(),
      Channels::Status => "status".to_string(),
      Channels::Ticker => "ticker".to_string(),
      Channels::Level2 => "level2".to_string(),
      Channels::Matches => "matches".to_string(),
      Channels::User => "user".to_string(),
      Channels::Full => "full".to_string(),
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Channel {
  name: Channels,
  product_ids: Option<Vec<String>>,
}

impl Channel {
  pub fn new(channel: Channels) -> Self { Channel { name: channel, product_ids: None } }

  pub fn with_product_ids(channel: Channels, product_ids: Vec<String>) -> Self {
    Channel { name: channel, product_ids: Some(product_ids) }
  }

  pub fn from_names(channels: &[Channels]) -> Vec<Self> {
    channels.iter()
      .map(|c| Channel::new(c.clone()))
      .collect()
  }
}

impl Serialize for Channel {
  fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
    S: Serializer {
    if self.product_ids.is_none() { // This is borrow
      serializer.serialize_str(&self.name.to_string())
    } else {
      let mut serializer = serializer.serialize_struct("channels", 2)?;
      serializer.serialize_field("name", &self.name)?;
      serializer.serialize_field("product_ids", &self.product_ids)?;
      serializer.end()
    }
  }
}

impl<'de> Deserialize<'de> for Channel {
  fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
    D: Deserializer<'de> {
    enum Field { Name, ProductIds }
    impl<'de> Deserialize<'de> for Field {
      fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: Deserializer<'de> {
        struct FieldVisitor;
        impl<'de> Visitor<'de> for FieldVisitor {
          type Value = Field;

          fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("`name` or `product_ids`")
          }

          fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where
            E: Error,
          {
            match v {
              "name" => Ok(Field::Name),
              "product_ids" => Ok(Field::ProductIds),
              _ => Err(Error::unknown_field(v, &["name", "product_ids"]))
            }
          }
        }

        deserializer.deserialize_identifier(FieldVisitor)
      }
    }

    struct ChannelVisitor;
    impl<'de> Visitor<'de> for ChannelVisitor {
      type Value = Channel;

      fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("struct Channel.")
      }

      fn visit_str<E>(self, name: &str) -> Result<Self::Value, E>
        where
          E: Error,
      {
        let name = FromStr::from_str(name)
          .map_err(|_err| serde::de::Error::custom(format!("could not parse {} into channel name.", name)))?;

        Ok(Channel { name, product_ids: None })
      }

      fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
        where
          V: MapAccess<'de>
      {
        let mut name: Option<String> = None;
        let mut product_ids: Option<Vec<String>> = None;
        while let Some(key) = map.next_key()? {
          match key {
            Field::Name => {
              if name.is_some() {
                return Err(Error::duplicate_field("name"));
              }
              name = Some(map.next_value()?);
            }
            Field::ProductIds => {
              if product_ids.is_some() {
                return Err(Error::duplicate_field("product_ids"));
              }
              product_ids = Some(map.next_value()?);
            }
          }
        };
        let name = name.ok_or_else(|| Error::missing_field("name"))?;
        let name = FromStr::from_str(name.as_str())
          .map_err(|_err| serde::de::Error::custom(format!("could not parse {} into channel name.", name)))?;

        Ok(Channel { name, product_ids })
      }
    }

    deserializer.deserialize_any(ChannelVisitor)
  }
}

#[cfg(test)]
mod test {
  use super::Channel;
  use crate::web_socket::common::Channels;

  #[test]
  fn serialize_channel() {
    let channel = Channel { name: Channels::Status, product_ids: Some(vec!["bar".into()]) };
    let res = serde_json::to_string(&channel).unwrap();
    println!("{}", res)
  }

  #[test]
  fn deserialize_channel() -> Result<(), serde_json::error::Error> {
    let json = "{\"name\": \"ticker\"}";
    let channel: Channel = serde_json::de::from_str(json)?;
    println!("{:?}", channel);
    Ok(())
  }

  #[test]
  fn deserialize_channel_with_only_name() -> Result<(), serde_json::error::Error> {
    let json = "\"level2\"";
    let channel: Channel = serde_json::de::from_str(json)?;
    println!("{:?}", channel);
    Ok(())
  }
}