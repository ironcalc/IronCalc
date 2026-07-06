use std::collections::HashMap;

use ironcalc::base::finance::provider::{FinanceError, Ticker};

use crate::finance::provider::FinanceProvider;

/// A mock finance provider that returns canned responses.
///
/// Use [`MockFinanceProvider::set`] to predefine values for specific
/// `(ticker, attribute)` pairs. Any query not registered will return
/// [`FinanceError::TickerNotFound`].
pub struct MockFinanceProvider {
    data: HashMap<(Ticker, String), Result<f64, FinanceError>>,
}

impl MockFinanceProvider {
    /// Create an empty mock.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Register a value for the given ticker and attribute.
    pub fn set(&mut self, ticker: &Ticker, attribute: &str, value: f64) {
        self.data
            .insert((ticker.clone(), attribute.to_string()), Ok(value));
    }

    /// Register an error for the given ticker and attribute.
    pub fn set_error(&mut self, ticker: &Ticker, attribute: &str, error: FinanceError) {
        self.data
            .insert((ticker.clone(), attribute.to_string()), Err(error));
    }
}

impl Default for MockFinanceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FinanceProvider for MockFinanceProvider {
    async fn fetch(&self, ticker: &Ticker, attribute: &str) -> Result<f64, FinanceError> {
        self.data
            .get(&(ticker.clone(), attribute.to_string()))
            .cloned()
            .unwrap_or_else(|| Err(FinanceError::TickerNotFound(ticker.symbol().to_string())))
    }
}
