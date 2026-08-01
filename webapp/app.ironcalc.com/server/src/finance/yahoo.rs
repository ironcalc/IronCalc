use crate::finance::provider::FinanceProvider;
use ironcalc::base::finance::provider::{FinanceError, Ticker};
use serde::Deserialize;

/// Yahoo Finance data provider
///
/// Uses Yahoo's chart API endpoint which covers US stocks,
/// European ETFs, forex, crypto, and more.
pub struct YahooFinanceProvider {
    client: reqwest::Client,
}

impl YahooFinanceProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Build a Yahoo symbol from a ticker + attribute.
    ///
    /// - Stock: passed through as-is (user includes exchange suffix like
    ///   `EUNL.DE` for XETRA, `IWDA.AS` for Euronext, or just `AAPL` for US).
    /// - Forex: `EURUSD` → `EURUSD=X`
    /// - Crypto: `BTC-USD`
    fn yahoo_symbol(ticker: &Ticker) -> String {
        match ticker {
            Ticker::Stock(s) => s.clone(),
            Ticker::Forex(s) => format!("{}=X", s.to_uppercase()),
            Ticker::Crypto(s) => s.clone(),
        }
    }

    async fn fetch_price(&self, yahoo_symbol: &str, attribute: &str) -> Result<f64, FinanceError> {
        let url = format!(
            "https://query2.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1d",
            yahoo_symbol
        );

        let resp = self
            .client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|e| FinanceError::NetworkError(e.to_string()))?;

        if resp.status().as_u16() == 404 {
            return Err(FinanceError::TickerNotFound(yahoo_symbol.to_string()));
        }
        if !resp.status().is_success() {
            return Err(FinanceError::NetworkError(format!(
                "HTTP {}",
                resp.status()
            )));
        }

        let body: ChartResponse = resp
            .json()
            .await
            .map_err(|e| FinanceError::ParseError(e.to_string()))?;

        let result = body
            .chart
            .result
            .first()
            .ok_or_else(|| FinanceError::TickerNotFound(yahoo_symbol.to_string()))?;

        match attribute {
            "price" => Ok(result.meta.regular_market_price),
            "open" | "high" | "low" | "close" | "volume" => {
                let quote =
                    result.indicators.quote.first().ok_or_else(|| {
                        FinanceError::ParseError("missing quote data".to_string())
                    })?;
                let values = match attribute {
                    "open" => &quote.open,
                    "high" => &quote.high,
                    "low" => &quote.low,
                    "close" => &quote.close,
                    "volume" => &quote.volume,
                    _ => unreachable!(),
                };
                values.iter().rev().find_map(|v| *v).ok_or_else(|| {
                    FinanceError::ParseError(format!("no {attribute} data available"))
                })
            }
            other => Err(FinanceError::ParseError(format!(
                "unsupported attribute: {other}"
            ))),
        }
    }
}

impl FinanceProvider for YahooFinanceProvider {
    async fn fetch(&self, ticker: &Ticker, attribute: &str) -> Result<f64, FinanceError> {
        let symbol = Self::yahoo_symbol(ticker);
        self.fetch_price(&symbol, attribute).await
    }
}

#[derive(Debug, Deserialize)]
struct ChartResponse {
    chart: Chart,
}

#[derive(Debug, Deserialize)]
struct Chart {
    result: Vec<ChartResult>,
}

#[derive(Debug, Deserialize)]
struct ChartResult {
    meta: ChartMeta,
    #[serde(default)]
    indicators: Indicators,
}

#[derive(Debug, Deserialize)]
struct ChartMeta {
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: f64,
}

#[derive(Debug, Default, Deserialize)]
struct Indicators {
    #[serde(default)]
    quote: Vec<Quote>,
}

#[derive(Debug, Default, Deserialize)]
struct Quote {
    #[serde(default)]
    open: Vec<Option<f64>>,
    #[serde(default)]
    high: Vec<Option<f64>>,
    #[serde(default)]
    low: Vec<Option<f64>>,
    #[serde(default)]
    close: Vec<Option<f64>>,
    #[serde(default)]
    volume: Vec<Option<f64>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yahoo_symbol_stock() {
        assert_eq!(
            YahooFinanceProvider::yahoo_symbol(&Ticker::Stock("AAPL".into())),
            "AAPL"
        );
        assert_eq!(
            YahooFinanceProvider::yahoo_symbol(&Ticker::Stock("EUNL.DE".into())),
            "EUNL.DE"
        );
    }

    #[test]
    fn test_yahoo_symbol_forex() {
        assert_eq!(
            YahooFinanceProvider::yahoo_symbol(&Ticker::Forex("EURUSD".into())),
            "EURUSD=X"
        );
        assert_eq!(
            YahooFinanceProvider::yahoo_symbol(&Ticker::Forex("GBPJPY".into())),
            "GBPJPY=X"
        );
    }

    #[test]
    fn test_yahoo_symbol_crypto() {
        assert_eq!(
            YahooFinanceProvider::yahoo_symbol(&Ticker::Crypto("BTC-USD".into())),
            "BTC-USD"
        );
        assert_eq!(
            YahooFinanceProvider::yahoo_symbol(&Ticker::Crypto("ETH-USDC".into())),
            "ETH-USDC"
        );
    }
}
