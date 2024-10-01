use crate::prelude::*;
use alloy_primitives::aliases::U24;
use alloy_primitives::{keccak256, Address, B256, U160};
use alloy_sol_types::SolValue;
use uniswap_sdk_core::prelude::{BaseCurrency, Currency as CurrencyTrait};
use uniswap_v3_sdk::prelude::*;

pub const DYANMIC_FEE_FLAG: u32 = 0x800000;

#[derive(Clone, Debug)]
pub struct PoolKey<I: TickIndex> {
    currency0: Address,
    currency1: Address,
    fee: u32,
    tick_spacing: I,
    hooks: Address,
}

/// Represents a V4 pool
#[derive(Clone, Debug)]
pub struct Pool<TP = NoTickDataProvider>
where
    TP: TickDataProvider,
{
    pub currency0: Currency,
    pub currency1: Currency,
    pub fee: u32,
    pub tick_spacing: TP::Index,
    pub sqrt_ratio_x96: U160,
    pub hooks: Address,
    pub liquidity: u128,
    pub tick_current: TP::Index,
    pub tick_data_provider: TP,
    pub pool_key: PoolKey<TP::Index>,
    pub pool_id: B256,
}

impl Pool {
    fn sort_currency(currency_a: &Currency, currency_b: &Currency) -> (Address, Address) {
        if currency_a.is_native() {
            (Address::ZERO, currency_b.address())
        } else if currency_b.is_native() {
            (Address::ZERO, currency_a.address())
        } else if sorts_before(currency_a, currency_b) {
            (currency_a.address(), currency_b.address())
        } else {
            (currency_b.address(), currency_a.address())
        }
    }

    pub fn get_pool_key<I: TickIndex>(
        currency_a: &Currency,
        currency_b: &Currency,
        fee: u32,
        tick_spacing: I,
        hooks: Address,
    ) -> PoolKey<I> {
        let (currency0_addr, currency1_addr) = Self::sort_currency(currency_a, currency_b);
        PoolKey {
            currency0: currency0_addr,
            currency1: currency1_addr,
            fee,
            tick_spacing,
            hooks,
        }
    }

    pub fn get_pool_id<I: TickIndex>(
        currency_a: &Currency,
        currency_b: &Currency,
        fee: u32,
        tick_spacing: I,
        hooks: Address,
    ) -> B256 {
        let (currency0_addr, currency1_addr) = Self::sort_currency(currency_a, currency_b);
        keccak256(
            (
                currency0_addr,
                currency1_addr,
                U24::from(fee),
                tick_spacing.to_i24(),
                hooks,
            )
                .abi_encode(),
        )
    }

    /// Constructs a pool
    ///
    /// ## Arguments
    ///
    /// * `currency_a`: One of the currencies in the pool
    /// * `currency_b`: The other currency in the pool
    /// * `fee`: The fee in hundredths of a bips of the input amount of every swap that is collected by the pool
    /// * `tick_spacing`: The tickSpacing of the pool
    /// * `hooks`: The address of the hook contract
    /// * `sqrt_ratio_x96`: The sqrt of the current ratio of amounts of currency1 to currency0
    /// * `liquidity`: The current value of in range liquidity
    pub fn new(
        currency_a: Currency,
        currency_b: Currency,
        fee: u32,
        tick_spacing: <NoTickDataProvider as TickDataProvider>::Index,
        hooks: Address,
        sqrt_ratio_x96: U160,
        liquidity: u128,
    ) -> Self {
        Self::new_with_tick_data_provider(
            currency_a,
            currency_b,
            fee,
            tick_spacing,
            hooks,
            sqrt_ratio_x96,
            liquidity,
            NoTickDataProvider,
        )
    }
}

impl<TP: TickDataProvider> Pool<TP> {
    /// Construct a pool with a tick data provider
    ///
    /// ## Arguments
    ///
    /// * `currency_a`: One of the currencies in the pool
    /// * `currency_b`: The other currency in the pool
    /// * `fee`: The fee in hundredths of a bips of the input amount of every swap that is collected by the pool
    /// * `tick_spacing`: The tickSpacing of the pool
    /// * `hooks`: The address of the hook contract
    /// * `sqrt_ratio_x96`: The sqrt of the current ratio of amounts of currency1 to currency0
    /// * `liquidity`: The current value of in range liquidity
    /// * `tick_data_provider`: A tick data provider that can return tick data
    pub fn new_with_tick_data_provider(
        currency_a: Currency,
        currency_b: Currency,
        fee: u32,
        tick_spacing: TP::Index,
        hooks: Address,
        sqrt_ratio_x96: U160,
        liquidity: u128,
        tick_data_provider: TP,
    ) -> Self {
        assert!(fee == DYANMIC_FEE_FLAG || fee < 1_000_000, "FEE");
        if fee == DYANMIC_FEE_FLAG {
            assert_ne!(hooks, Address::ZERO, "Dynamic fee pool requires a hook");
        }
        let pool_key = Pool::get_pool_key(&currency_a, &currency_b, fee, tick_spacing, hooks);
        let pool_id = Pool::get_pool_id(&currency_a, &currency_b, fee, tick_spacing, hooks);
        let tick_current = sqrt_ratio_x96
            .get_tick_at_sqrt_ratio()
            .unwrap()
            .as_i32()
            .try_into()
            .unwrap();
        let (currency0, currency1) = if sorts_before(&currency_a, &currency_b) {
            (currency_a, currency_b)
        } else {
            (currency_b, currency_a)
        };
        Self {
            currency0,
            currency1,
            fee,
            tick_spacing,
            sqrt_ratio_x96,
            hooks,
            liquidity,
            tick_current,
            tick_data_provider,
            pool_key,
            pool_id,
        }
    }
}
