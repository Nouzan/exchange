use exc::{
    service::fetch_candles::BackwardCandles,
    transport::http::endpoint::Endpoint,
    types::candle::{Period, QueryCandles},
    Exchange,
};
use exc_okx::http::{layer::OkxHttpApiLayer, types::request::HttpRequest};
use futures::StreamExt;
use time::macros::{datetime, offset};
use tower::{ServiceBuilder, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "okx_candle=debug,exc_okx=debug".into()),
        ))
        .init();
    tracing::debug!("hello world");
    let channel = Endpoint::default().connect_https();
    let svc = ServiceBuilder::default()
        .layer(OkxHttpApiLayer::default())
        .service(channel);
    let svc = Exchange::<_, HttpRequest>::new(svc);
    let mut svc = ServiceBuilder::default()
        .layer(BackwardCandles::new(100, 2))
        .rate_limit(10, std::time::Duration::from_secs(2))
        .service(svc);
    let query = QueryCandles::new(
        "BTC-USDT",
        Period::minutes(offset!(+8), 1),
        datetime!(2022-01-01 00:00:00 +08:00)..,
    );
    let mut stream = (&mut svc).oneshot(query).await?;
    while let Some(c) = stream.next().await {
        match c {
            Ok(c) => tracing::info!("{c}"),
            Err(err) => tracing::error!("{err}"),
        }
    }
    Ok(())
}
