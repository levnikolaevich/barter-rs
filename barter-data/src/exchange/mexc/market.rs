use crate::{instrument::MarketInstrumentData, subscription::Subscription, Identifier};

use super::Mexc;
use barter_instrument::{
    Keyed, asset::name::AssetNameInternal, instrument::market_data::MarketDataInstrument,
};
use serde::{Deserialize, Serialize};
use smol_str::{SmolStr, StrExt, format_smolstr};

pub mod proto {
    include!("protobuf_gen/_.rs");
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct MexcMarket(pub SmolStr);

impl<Kind> Identifier<MexcMarket> for Subscription<Mexc, MarketDataInstrument, Kind> {
    fn id(&self) -> MexcMarket {
        mexc_market(&self.instrument.base, &self.instrument.quote)
    }
}

impl<InstrumentKey, Kind> Identifier<MexcMarket>
    for Subscription<Mexc, Keyed<InstrumentKey, MarketDataInstrument>, Kind>
{
    fn id(&self) -> MexcMarket {
        mexc_market(&self.instrument.value.base, &self.instrument.value.quote)
    }
}

impl<InstrumentKey, Kind> Identifier<MexcMarket>
    for Subscription<Mexc, MarketInstrumentData<InstrumentKey>, Kind>
{
    fn id(&self) -> MexcMarket {
        MexcMarket(self.instrument.name_exchange.name().clone())
    }
}

impl AsRef<str> for MexcMarket {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

fn mexc_market(base: &AssetNameInternal, quote: &AssetNameInternal) -> MexcMarket {
    MexcMarket(format_smolstr!("{base}{quote}").to_uppercase_smolstr())
}