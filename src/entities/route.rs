use crate::prelude::*;
use alloc::vec::Vec;
use alloy_primitives::ChainId;
use uniswap_sdk_core::prelude::{BaseCurrency, Currency, Price};
use uniswap_v3_sdk::entities::TickDataProvider;

/// Represents a list of pools through which a swap can occur
#[derive(Clone, PartialEq, Debug)]
pub struct Route<TInput, TOutput, TP>
where
    TInput: BaseCurrency,
    TOutput: BaseCurrency,
    TP: TickDataProvider,
{
    pub pools: Vec<Pool<TP>>,
    /// The input currency
    pub input: TInput,
    /// The output currency
    pub output: TOutput,
    /// equivalent or wrapped/unwrapped input to match pool
    pub path_input: Currency,
    /// equivalent or wrapped/unwrapped output to match pool
    pub path_output: Currency,
    _mid_price: Option<Price<TInput, TOutput>>,
}

impl<TInput, TOutput, TP> Route<TInput, TOutput, TP>
where
    TInput: BaseCurrency,
    TOutput: BaseCurrency,
    TP: TickDataProvider,
{
    /// Creates an instance of route.
    ///
    /// ## Arguments
    ///
    /// * `pools`: An array of [`Pool`] objects, ordered by the route the swap will take
    /// * `input`: The input currency
    /// * `output`: The output currency
    #[inline]
    pub fn new(pools: Vec<Pool<TP>>, input: TInput, output: TOutput) -> Result<Self, Error> {
        assert!(!pools.is_empty(), "POOLS");

        let chain_id = pools[0].chain_id();
        let all_on_same_chain = pools.iter().all(|pool| pool.chain_id() == chain_id);
        assert!(all_on_same_chain, "CHAIN_IDS");

        // throws if pools do not involve the input and output currency or the native/wrapped
        // equivalent
        let path_input = get_path_currency(&input, &pools[0])?;
        let path_output = get_path_currency(&output, pools.last().unwrap())?;

        let mut current_input_currency = &path_input;
        for pool in &pools {
            current_input_currency = if current_input_currency.equals(&pool.currency0) {
                &pool.currency1
            } else if current_input_currency.equals(&pool.currency1) {
                &pool.currency0
            } else {
                panic!("PATH")
            };
        }
        assert!(current_input_currency.equals(&path_output), "PATH");

        Ok(Self {
            pools,
            input,
            output,
            path_input,
            path_output,
            _mid_price: None,
        })
    }

    /// Normalizes currency0-currency1 order and selects the next currency/fee step to add to the
    /// path
    #[inline]
    pub fn currency_path(&self) -> Vec<Currency> {
        let mut currency_path: Vec<Currency> = Vec::with_capacity(self.pools.len() + 1);
        currency_path.push(self.path_input.clone());
        for (i, pool) in self.pools.iter().enumerate() {
            let next_currency = if currency_path[i].equals(&pool.currency0) {
                pool.currency1.clone()
            } else {
                pool.currency0.clone()
            };
            currency_path.push(next_currency);
        }
        currency_path
    }

    #[inline]
    pub fn chain_id(&self) -> ChainId {
        self.pools[0].chain_id()
    }

    /// Returns the mid price of the route
    #[inline]
    pub fn mid_price(&self) -> Result<Price<TInput, TOutput>, Error> {
        let mut price = self.pools[0].price_of(&self.path_input)?;
        for pool in &self.pools[1..] {
            price = price.multiply(&pool.price_of(&price.quote_currency)?)?;
        }
        Ok(Price::new(
            self.input.clone(),
            self.output.clone(),
            price.denominator,
            price.numerator,
        ))
    }

    /// Returns the cached mid price of the route
    #[inline]
    pub fn mid_price_cached(&mut self) -> Result<Price<TInput, TOutput>, Error> {
        if let Some(mid_price) = &self._mid_price {
            return Ok(mid_price.clone());
        }
        let mid_price = self.mid_price()?;
        self._mid_price = Some(mid_price.clone());
        Ok(mid_price)
    }
}
