use super::Mexc;
use crate::{
    subscription::{trade::PublicTrades, Subscription},
    Identifier,
};
use serde::Serialize;

/// Defines how to translate a Barter [`Subscription`] into an [`MexcChannel`]
/// base string for WebSocket subscriptions.
///
/// The actual subscription topic sent to MEXC for aggregated deals will be
/// dynamically constructed by appending "@<interval>@<symbol>" to this base channel string.
/// For example: "spot@public.aggre.deals.v3.api.pb@100ms@BTC_USDT".
///
/// Important: This channel uses Protocol Buffers (.pb) for data format.
///
/// MEXC WebSocket API (Spot V3) Documentation:
/// - Trade Streams: <https://mexcdevelop.github.io/apidocs/spot_v3_en/#trade-streams>
/// - Spot aggregated deals stream: Referenced within the "Trade Streams" section.
/// - Public Subscription Method: <https://mexcdevelop.github.io/apidocs/spot_v3_en/#public-subscription>
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize)]
pub struct MexcChannel(pub &'static str);

impl MexcChannel {
    /// Base channel string for [`Mexc`]'s real-time public aggregated deals (trades)
    /// stream using Protocol Buffers.
    ///
    /// The specific aggregation interval (e.g., "100ms") and market symbol
    /// (e.g., "BTC_USDT") will be appended to this string (prefixed with "@")
    /// when forming the actual subscription message.
    ///
    /// Example base string: "spot@public.aggre.deals.v3.api.pb"
    pub const AGGREGATED_TRADES_PB: Self = Self("spot@public.aggre.deals.v3.api.pb");
}

impl<Instrument> Identifier<MexcChannel> for Subscription<Mexc, Instrument, PublicTrades> {
    fn id(&self) -> MexcChannel {
        // Default to using aggregated trades in Protobuf format.
        // The choice of interval (100ms/10ms) will be handled in the subscription logic.
        MexcChannel::AGGREGATED_TRADES_PB
    }
}

impl AsRef<str> for MexcChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}
