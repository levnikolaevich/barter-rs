use self::{
    channel::MexcChannel,
    market::MexcMarket,
    subscription::{MexcAggInterval, MexcWsMethod, MexcWsSub},
};
use crate::{
    ExchangeWsStream, Identifier, NoInitialSnapshots,
    exchange::{Connector, ExchangeSub, PingInterval, StreamSelector},
    instrument::InstrumentData,
    subscriber::{WebSocketSubscriber, validator::WebSocketSubValidator},
    subscription::{Map, trade::PublicTrades},
    transformer::stateless::StatelessTransformer,
};
use barter_instrument::exchange::ExchangeId;
use barter_integration::{
    error::SocketError, protocol::websocket::WsMessage, subscription::SubscriptionId,
};
use barter_macro::{DeExchange, SerExchange};
use derive_more::Display;
use serde::Deserialize;
use std::{borrow::Cow, time::Duration};
use tokio::time;
use url::Url;

pub mod channel;
pub mod market;
pub mod subscription;
pub mod trade;

/// MEXC WebSocket API base URL for public market data streams (Secure).
/// Docs: <https://mexcdevelop.github.io/apidocs/spot_v3_en/#websocket-market-data>
pub const BASE_URL_MEXC: &str = "wss://wbs.mexc.com/ws";

/// [`Mexc`] exchange connector definition.
///
/// MEXC uses Protocol Buffers for its V3 WebSocket API for public data streams like trades.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Default,
    Display,
    DeExchange,
    SerExchange,
)]
pub struct Mexc;

impl Mexc {
    fn agg_interval_to_str(interval: MexcAggInterval) -> &'static str {
        match interval {
            MexcAggInterval::Ms10 => "10ms",
            MexcAggInterval::Ms100 => "100ms",
        }
    }
}

impl Connector for Mexc {
    const ID: ExchangeId = ExchangeId::Mexc;
    type Channel = MexcChannel;
    type Market = MexcMarket;
    type Subscriber = WebSocketSubscriber;
    type SubValidator = WebSocketSubValidator;
    type SubResponse = self::subscription::MexcSubResponse;

    fn url() -> Result<Url, SocketError> {
        Url::parse(BASE_URL_MEXC).map_err(SocketError::UrlParse)
    }

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            interval: time::interval(Duration::from_secs(10)),
            ping: || WsMessage::Text("ping".to_string().into()),
        })
    }

    fn requests(exchange_subs: Vec<ExchangeSub<Self::Channel, Self::Market>>) -> Vec<WsMessage> {
        if exchange_subs.is_empty() {
            return Vec::new();
        }
        let default_interval = MexcAggInterval::default();

        let topics = exchange_subs
            .into_iter()
            .map(|sub| {
                format!(
                    "{}@{}@{}",
                    sub.channel.0,
                    Self::agg_interval_to_str(default_interval),
                    sub.market.0
                )
            })
            .collect::<Vec<String>>();

        let subscription_message = MexcWsSub {
            method: MexcWsMethod::Subscription,
            params: Cow::Owned(topics),
        };

        match serde_json::to_string(&subscription_message) {
            Ok(text_payload) => vec![WsMessage::Text(text_payload.into())],
            Err(e) => {
                eprintln!("Failed to serialize MEXC subscription request: {}", e);
                Vec::new()
            }
        }
    }

    fn expected_responses<InstrumentKey>(subscriptions: &Map<InstrumentKey>) -> usize {
        subscriptions.0.len()
    }
}

// Stub `Deserialize` implementation to satisfy trait bounds.
// MEXC V3 uses Protocol Buffers, so proper decoding should be handled
// in the WebSocket layer rather than via Serde text deserialisation.
impl<'de> Deserialize<'de> for self::trade::proto::PushDataV3ApiWrapper {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Err(serde::de::Error::custom(
            "Attempted to Deserialize PushDataV3ApiWrapper with Serde text deserializer. \
            MEXC V3 uses Protobuf binary format. Implement proper Protobuf deserialization in the WebSocket handling layer.",
        ))
    }
}

impl Identifier<Option<SubscriptionId>> for self::trade::proto::PushDataV3ApiWrapper {
    fn id(&self) -> Option<SubscriptionId> {
        // Use the raw channel string as the subscription identifier.
        // This may be adjusted once a dedicated Protobuf parser is implemented.
        Some(SubscriptionId::from(self.channel.as_str()))
    }
}

impl<Instrument> StreamSelector<Instrument, PublicTrades> for Mexc
where
    Instrument: InstrumentData,
{
    type SnapFetcher = NoInitialSnapshots;
    type Stream = ExchangeWsStream<
        StatelessTransformer<
            Self,
            Instrument::Key,
            PublicTrades,
            self::trade::proto::PushDataV3ApiWrapper,
        >,
    >;
}
