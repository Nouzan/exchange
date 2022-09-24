use exc::{
    transport::http::endpoint::Endpoint as HttpEndpoint,
    types::trading::{Place, PlaceOrderOptions},
    ExcLayer, {CheckOrderService, TradingService},
};
use exc_okx::{
    http::{layer::OkxHttpApiLayer, types::request::HttpRequest},
    key::Key,
    websocket::Endpoint,
};
use rust_decimal_macros::dec;
use std::env::var;
use tower::ServiceBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,okx_ws_trading_testing=debug,exc_okx=trace".into()),
        ))
        .init();

    let key = Key {
        apikey: var("OKX_APIKEY")?,
        secretkey: var("OKX_SECRETKEY")?,
        passphrase: var("OKX_PASSPHRASE")?,
    };

    let channel = Endpoint::default()
        .testing(true)
        .request_timeout(std::time::Duration::from_secs(5))
        .private(key.clone())
        .connect();
    let mut ws = ServiceBuilder::default()
        .layer(ExcLayer::default())
        .service(channel);

    let mut http = ServiceBuilder::default()
        .rate_limit(59, std::time::Duration::from_secs(2))
        .layer(ExcLayer::<HttpRequest>::default())
        .layer(OkxHttpApiLayer::default().private(key).testing(true))
        .service(HttpEndpoint::default().connect_https());

    let inst = "DOGE-USDT";
    let id = ws
        .place_with_opts(
            &Place::with_size(dec!(10)).post_only(dec!(0.01)),
            &PlaceOrderOptions::new(inst),
        )
        .await?
        .id;
    tracing::info!("id={id:?}");
    let order = http.check(inst, &id).await?;
    tracing::info!("order={order:?}");
    ws.cancel(inst, &id).await?;
    tracing::info!("cancelled");
    let order = http.check(inst, &id).await?;
    tracing::info!("order={order:?}");
    Ok(())
}
