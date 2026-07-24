use ironcalc::base::finance::provider::{FinanceError, Ticker};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// TTL for each asset class.
const STOCK_TTL: Duration = Duration::from_secs(60);
const FOREX_TTL: Duration = Duration::from_secs(300);
const CRYPTO_TTL: Duration = Duration::from_secs(300);

/// A cached result entry.
#[derive(Clone)]
struct CacheEntry {
    value: Result<f64, FinanceError>,
    fetched_at: Instant,
}

/// In-memory cache for financial data fetches.
pub struct Cache {
    entries: HashMap<(Ticker, String), CacheEntry>,
}

impl Cache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Look up a cached value. Returns `None` if not present or expired.
    pub fn get(&self, ticker: &Ticker, attribute: &str) -> Option<&Result<f64, FinanceError>> {
        let entry = self.entries.get(&(ticker.clone(), attribute.to_string()))?;
        let ttl = ttl_for(ticker);
        if entry.fetched_at.elapsed() > ttl {
            return None;
        }
        Some(&entry.value)
    }

    /// Insert a value into the cache.
    pub fn insert(&mut self, ticker: &Ticker, attribute: &str, value: Result<f64, FinanceError>) {
        self.entries.insert(
            (ticker.clone(), attribute.to_string()),
            CacheEntry {
                value,
                fetched_at: Instant::now(),
            },
        );
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

/// Return the TTL for a given ticker's asset class.
fn ttl_for(ticker: &Ticker) -> Duration {
    match ticker {
        Ticker::Stock(_) => STOCK_TTL,
        Ticker::Forex(_) => FOREX_TTL,
        Ticker::Crypto(_) => CRYPTO_TTL,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit_and_miss() {
        let mut cache = Cache::new();

        let ticker = Ticker::Stock("AAPL".to_string());
        assert!(cache.get(&ticker, "price").is_none());

        cache.insert(&ticker, "price", Ok(195.89));
        assert_eq!(
            cache.get(&ticker, "price").unwrap().clone().unwrap(),
            195.89
        );

        // Different attribute should miss
        assert!(cache.get(&ticker, "open").is_none());
    }

    #[test]
    fn test_cache_with_error() {
        let mut cache = Cache::new();
        let ticker = Ticker::Stock("BADTICKER".to_string());

        cache.insert(
            &ticker,
            "price",
            Err(FinanceError::TickerNotFound("BADTICKER".into())),
        );
        assert!(matches!(
            cache.get(&ticker, "price"),
            Some(Err(FinanceError::TickerNotFound(_)))
        ));
    }

    #[test]
    fn test_different_ttls() {
        // Just verify TTLs are correctly assigned
        assert_eq!(
            ttl_for(&Ticker::Stock("AAPL".into())),
            Duration::from_secs(60)
        );
        assert_eq!(
            ttl_for(&Ticker::Forex("EURUSD".into())),
            Duration::from_secs(300)
        );
        assert_eq!(
            ttl_for(&Ticker::Crypto("BTCUSDT".into())),
            Duration::from_secs(300)
        );
    }
}
