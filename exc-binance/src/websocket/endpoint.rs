use std::{collections::HashSet, time::Duration};

use futures::FutureExt;
use tower::{reconnect::Reconnect, ServiceExt};

use crate::{http::response::ListenKey, types::Name};

use super::connect::{BinanceWsConnect, BinanceWsTarget};
use super::{error::WsError, protocol::WsClient, request::WsRequest, BinanceWebsocketApi};

const DEFAULT_KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_STREAM_TIMEOUT: Duration = Duration::from_secs(30);

/// A builder of binance websocket api service.
#[derive(Debug, Clone)]
pub struct WsEndpoint {
    target: BinanceWsTarget,
    main_stream: HashSet<Name>,
    keep_alive_timeout: Option<Duration>,
    default_stream_timeout: Option<Duration>,
}

impl WsEndpoint {
    /// Create a new binance websocket api endpoint.
    pub fn new(host: String, name: Name) -> Self {
        Self {
            target: BinanceWsTarget { host, name },
            main_stream: HashSet::default(),
            keep_alive_timeout: None,
            default_stream_timeout: None,
        }
    }

    /// Set the keep-alive timeout.
    pub fn keep_alive_timeout(&mut self, duration: Duration) -> &mut Self {
        self.keep_alive_timeout = Some(duration);
        self
    }

    /// Set the default stream timeout for each request stream.
    pub fn default_stream_timeout(&mut self, duration: Duration) -> &mut Self {
        self.default_stream_timeout = Some(duration);
        self
    }

    // /// Private endpoint of USD-M Futures API.
    // pub fn private(&mut self, listen_key: ListenKey) -> Result<&mut Self, WsError> {
    //     let uri = Uri::from_str(format!("wss://fstream.binance.com/ws/{listen_key}").as_str())?;
    //     self.uri = uri;
    //     Ok(self)
    // }

    /// Add main stream.
    pub(crate) fn add_main_stream(&mut self, name: Name) -> &mut Self {
        self.main_stream.insert(name);
        self
    }

    /// Connect to binance websocket api.
    pub fn connect(&self) -> BinanceWebsocketApi {
        let main_stream = self.main_stream.clone();
        let keep_alive_timeout = self
            .keep_alive_timeout
            .unwrap_or(DEFAULT_KEEP_ALIVE_TIMEOUT);
        let default_stream_timeout = self
            .default_stream_timeout
            .unwrap_or(DEFAULT_STREAM_TIMEOUT);
        let connect = BinanceWsConnect::default().and_then(move |ws| {
            async move {
                WsClient::with_websocket(
                    ws,
                    main_stream,
                    keep_alive_timeout,
                    default_stream_timeout,
                )
            }
            .boxed()
        });
        let connection = Reconnect::new::<WsClient, WsRequest>(connect, self.target.clone())
            .map_err(|err| match err.downcast::<WsError>() {
                Ok(err) => *err,
                Err(err) => WsError::UnknownConnection(err),
            });
        BinanceWebsocketApi {
            svc: connection.boxed(),
        }
    }
}
