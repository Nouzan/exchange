use exc_core::{
    types::instrument::{Attributes, FetchInstruments, InstrumentMeta},
    Adaptor, ExchangeError,
};
use futures::{stream, StreamExt};
use rust_decimal::Decimal;

use crate::{
    http::{
        request::{self, Payload, RestRequest},
        response::{
            self,
            instrument::{Filter, SymbolFilter},
        },
    },
    Request,
};

impl Adaptor<FetchInstruments> for Request {
    fn from_request(_req: FetchInstruments) -> Result<Self, ExchangeError>
    where
        Self: Sized,
    {
        Ok(Request::Http(RestRequest::from(Payload::new(
            request::ExchangeInfo,
        ))))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<FetchInstruments as exc_core::Request>::Response, ExchangeError> {
        let info = resp.into_response::<response::ExchangeInfo>()?;
        match info {
            response::ExchangeInfo::UsdMarginFutures(info) => {
                Ok(stream::iter(info.symbols.into_iter().filter_map(|symbol| {
                    let mut price_tick = None;
                    let mut size_tick = None;
                    let mut min_size = None;
                    let mut min_value = None;
                    for filter in &symbol.filters {
                        if let Filter::Symbol(filter) = filter {
                            match filter {
                                SymbolFilter::PriceFilter { tick_size, .. } => {
                                    price_tick = Some(tick_size.normalize());
                                }
                                SymbolFilter::LotSize {
                                    min_qty, step_size, ..
                                } => {
                                    min_size = Some(min_qty.normalize());
                                    size_tick = Some(step_size.normalize());
                                }
                                SymbolFilter::MinNotional { notional } => {
                                    min_value = Some(notional);
                                }
                                _ => {}
                            }
                        }
                    }
                    let attrs = Attributes {
                        reversed: false,
                        unit: Decimal::ONE,
                        price_tick: price_tick?,
                        size_tick: size_tick?,
                        min_size: min_size?,
                        min_value: min_value.copied()?,
                    };
                    Some(Ok(InstrumentMeta::new(
                        symbol.symbol.to_lowercase(),
                        symbol
                            .to_exc_symbol()
                            .map_err(|err| {
                                tracing::error!(%err, "cannot build exc symbol from {}", symbol.symbol);
                                err
                            })
                            .ok()?,
                        attrs,
                    )))
                }))
                .boxed())
            }
            response::ExchangeInfo::Spot(info) => {
                Ok(stream::iter(info.symbols.into_iter().filter_map(|symbol| {
                    let mut price_tick = None;
                    let mut size_tick = None;
                    let mut min_size = None;
                    let mut min_value = None;
                    for filter in &symbol.filters {
                        if let Filter::Symbol(filter) = filter {
                            match filter {
                                SymbolFilter::PriceFilter { tick_size, .. } => {
                                    price_tick = Some(tick_size.normalize());
                                }
                                SymbolFilter::LotSize {
                                    min_qty, step_size, ..
                                } => {
                                    min_size = Some(min_qty.normalize());
                                    size_tick = Some(step_size.normalize());
                                }
                                SymbolFilter::Notional { min_notional } => {
                                    min_value = Some(min_notional.normalize());
                                }
                                _ => {}
                            }
                        }
                    }
                    tracing::debug!("{price_tick:?} {size_tick:?} {min_size:?} {min_value:?}");
                    let attrs = Attributes {
                        reversed: false,
                        unit: Decimal::ONE,
                        price_tick: price_tick?,
                        size_tick: size_tick?,
                        min_size: min_size?,
                        min_value: min_value?,
                    };
                    Some(Ok(InstrumentMeta::new(
                        symbol.symbol.to_lowercase(),
                        symbol
                            .to_exc_symbol()
                            .map_err(|err| {
                                tracing::error!(%err, "cannot build exc symbol from {}", symbol.symbol);
                                err
                            })
                            .ok()?,
                        attrs,
                    )))
                }))
                .boxed())
            }
        }
    }
}
