use serde::{Deserialize, Serialize};
use std::fmt;

/// Identifies the asset class of a ticker so providers can route to the
/// correct API endpoint without guessing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ticker {
    /// A stock or ETF ticker (e.g. `AAPL`, `MSFT`, `SPY`).
    Stock(String),
    /// A forex pair (e.g. `EURUSD`, `GBPJPY`).
    Forex(String),
    /// A cryptocurrency pair (e.g. `BTCUSDT`, `ETHBTC`).
    Crypto(String),
}

impl Ticker {
    /// Return the raw symbol string regardless of variant.
    pub fn symbol(&self) -> &str {
        match self {
            Ticker::Stock(s) | Ticker::Forex(s) | Ticker::Crypto(s) => s.as_str(),
        }
    }
}

/// Errors that can occur when fetching financial data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinanceError {
    /// The ticker was not found (404 or empty result).
    TickerNotFound(String),
    /// API key is missing or invalid (401/403).
    ApiKeyMissing,
    /// HTTP-level failure (DNS, TLS, timeout, connection refused).
    NetworkError(String),
    /// Rate limited by the provider (429).
    RateLimited,
    /// Response was received but JSON parsing failed.
    ParseError(String),
}

impl fmt::Display for FinanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FinanceError::TickerNotFound(ticker) => write!(f, "ticker not found: {ticker}"),
            FinanceError::ApiKeyMissing => write!(f, "API key missing or invalid"),
            FinanceError::NetworkError(msg) => write!(f, "network error: {msg}"),
            FinanceError::RateLimited => write!(f, "rate limited by provider"),
            FinanceError::ParseError(msg) => write!(f, "parse error: {msg}"),
        }
    }
}
