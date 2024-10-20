use crate::prelude::*;
use uniswap_sdk_core::prelude::{BigInt, Currency, CurrencyAmount, Price, Zero};
use uniswap_v3_sdk::prelude::{
    get_amount_0_delta, get_amount_1_delta, get_sqrt_ratio_at_tick, MintAmounts,
    NoTickDataProvider, TickDataProvider, TickIndex, ToBig, MAX_TICK, MIN_TICK,
};

/// Represents a position on a Uniswap V4 Pool
#[derive(Clone, Debug)]
pub struct Position<TP = NoTickDataProvider>
where
    TP: TickDataProvider,
{
    pub pool: Pool<TP>,
    pub tick_lower: TP::Index,
    pub tick_upper: TP::Index,
    pub liquidity: u128,
    _token0_amount: Option<CurrencyAmount<Currency>>,
    _token1_amount: Option<CurrencyAmount<Currency>>,
    _mint_amounts: Option<MintAmounts>,
}

impl<TP: TickDataProvider> Position<TP> {
    /// Constructs a position for a given pool with the given liquidity
    ///
    /// ## Arguments
    ///
    /// * `pool`: For which pool the liquidity is assigned
    /// * `liquidity`: The amount of liquidity that is in the position
    /// * `tick_lower`: The lower tick of the position
    /// * `tick_upper`: The upper tick of the position
    #[inline]
    pub fn new(
        pool: Pool<TP>,
        liquidity: u128,
        tick_lower: TP::Index,
        tick_upper: TP::Index,
    ) -> Self {
        assert!(tick_lower < tick_upper, "TICK_ORDER");
        assert!(
            tick_lower >= TP::Index::from_i24(MIN_TICK)
                && (tick_lower % pool.tick_spacing).is_zero(),
            "TICK_LOWER"
        );
        assert!(
            tick_upper <= TP::Index::from_i24(MAX_TICK)
                && (tick_upper % pool.tick_spacing).is_zero(),
            "TICK_UPPER"
        );
        Self {
            pool,
            liquidity,
            tick_lower,
            tick_upper,
            _token0_amount: None,
            _token1_amount: None,
            _mint_amounts: None,
        }
    }

    /// Returns the price of token0 at the lower tick
    #[inline]
    pub fn token0_price_lower(&self) -> Result<Price<Currency, Currency>, Error> {
        tick_to_price(
            self.pool.currency0.clone(),
            self.pool.currency1.clone(),
            self.tick_lower.to_i24(),
        )
    }

    /// Returns the price of token0 at the upper tick
    #[inline]
    pub fn token0_price_upper(&self) -> Result<Price<Currency, Currency>, Error> {
        tick_to_price(
            self.pool.currency0.clone(),
            self.pool.currency1.clone(),
            self.tick_upper.to_i24(),
        )
    }

    /// Returns the amount of token0 that this position's liquidity could be burned for at the
    /// current pool price
    #[inline]
    pub fn amount0(&self) -> Result<CurrencyAmount<Currency>, Error> {
        if self.pool.tick_current < self.tick_lower {
            CurrencyAmount::from_raw_amount(
                self.pool.currency0.clone(),
                get_amount_0_delta(
                    get_sqrt_ratio_at_tick(self.tick_lower.to_i24())?,
                    get_sqrt_ratio_at_tick(self.tick_upper.to_i24())?,
                    self.liquidity,
                    false,
                )?
                .to_big_int(),
            )
        } else if self.pool.tick_current < self.tick_upper {
            CurrencyAmount::from_raw_amount(
                self.pool.currency0.clone(),
                get_amount_0_delta(
                    self.pool.sqrt_ratio_x96,
                    get_sqrt_ratio_at_tick(self.tick_upper.to_i24())?,
                    self.liquidity,
                    false,
                )?
                .to_big_int(),
            )
        } else {
            CurrencyAmount::from_raw_amount(self.pool.currency0.clone(), BigInt::zero())
        }
        .map_err(Error::Core)
    }

    /// Returns the amount of token0 that this position's liquidity could be burned for at the
    /// current pool price
    #[inline]
    pub fn amount0_cached(&mut self) -> Result<CurrencyAmount<Currency>, Error> {
        if let Some(amount) = &self._token0_amount {
            return Ok(amount.clone());
        }
        let amount = self.amount0()?;
        self._token0_amount = Some(amount.clone());
        Ok(amount)
    }

    /// Returns the amount of token1 that this position's liquidity could be burned for at the
    /// current pool price
    #[inline]
    pub fn amount1(&self) -> Result<CurrencyAmount<Currency>, Error> {
        if self.pool.tick_current < self.tick_lower {
            CurrencyAmount::from_raw_amount(self.pool.currency1.clone(), BigInt::zero())
        } else if self.pool.tick_current < self.tick_upper {
            CurrencyAmount::from_raw_amount(
                self.pool.currency1.clone(),
                get_amount_1_delta(
                    get_sqrt_ratio_at_tick(self.tick_lower.to_i24())?,
                    self.pool.sqrt_ratio_x96,
                    self.liquidity,
                    false,
                )?
                .to_big_int(),
            )
        } else {
            CurrencyAmount::from_raw_amount(
                self.pool.currency1.clone(),
                get_amount_1_delta(
                    get_sqrt_ratio_at_tick(self.tick_lower.to_i24())?,
                    get_sqrt_ratio_at_tick(self.tick_upper.to_i24())?,
                    self.liquidity,
                    false,
                )?
                .to_big_int(),
            )
        }
        .map_err(Error::Core)
    }

    /// Returns the amount of token1 that this position's liquidity could be burned for at the
    /// current pool price
    #[inline]
    pub fn amount1_cached(&mut self) -> Result<CurrencyAmount<Currency>, Error> {
        if let Some(amount) = &self._token1_amount {
            return Ok(amount.clone());
        }
        let amount = self.amount1()?;
        self._token1_amount = Some(amount.clone());
        Ok(amount)
    }
}
