use ironcalc::base::finance::provider::{FinanceError, Ticker};

/// Trait for financial data providers.
///
/// Implementations fetch real-time or historical quotes for stocks, forex, and crypto.
pub trait FinanceProvider {
    /// Fetch a single numeric attribute for the given ticker.
    ///
    /// `attribute` is a string key like `"price"`, `"open"`, `"high"`,
    /// `"low"`, `"close"`, `"volume"`.
    async fn fetch(&self, ticker: &Ticker, attribute: &str) -> Result<f64, FinanceError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finance::{cache::Cache, mock::MockFinanceProvider};

    /// Full integration test: cache miss → provider fetch → cache insert → cache hit.
    #[tokio::test]
    async fn test_provider_with_cache() {
        let mut mock = MockFinanceProvider::new();
        mock.set(&Ticker::Stock("AAPL".into()), "price", 195.89);
        mock.set(&Ticker::Forex("EURUSD".into()), "price", 1.0850);

        let mut cache = Cache::new();

        // Cache miss — fetch from provider
        let ticker = Ticker::Stock("AAPL".to_string());
        let result = if let Some(cached) = cache.get(&ticker, "price") {
            cached.clone()
        } else {
            let r = mock.fetch(&ticker, "price").await;
            cache.insert(&ticker, "price", r.clone());
            r
        };
        assert_eq!(result.unwrap(), 195.89);

        // Cache hit — should return the same value without provider
        let cached = cache.get(&ticker, "price");
        assert_eq!(cached.unwrap().clone().unwrap(), 195.89);

        // Different ticker — cache miss
        let forex = Ticker::Forex("EURUSD".to_string());
        assert!(cache.get(&forex, "price").is_none());
        let result = mock.fetch(&forex, "price").await;
        assert_eq!(result.unwrap(), 1.0850);

        // Unknown ticker — provider returns error
        let unknown = Ticker::Stock("BADTICKER".to_string());
        let result = mock.fetch(&unknown, "price").await;
        assert!(matches!(result, Err(FinanceError::TickerNotFound(_))));
    }

    /// Verify that error results are also cached.
    #[tokio::test]
    async fn test_cache_stores_errors() {
        let mock = MockFinanceProvider::new(); // empty → all queries fail
        let mut cache = Cache::new();

        let ticker = Ticker::Stock("NONEXISTENT".to_string());

        // Fetch (will error)
        let result = mock.fetch(&ticker, "price").await;
        assert!(result.is_err());
        cache.insert(&ticker, "price", result.clone());

        // Should be cached now
        let cached = cache.get(&ticker, "price");
        assert!(cached.is_some());
        assert!(cached.unwrap().is_err());
    }
}
