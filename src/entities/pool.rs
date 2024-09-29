use crate::prelude::*;
use alloy_primitives::aliases::U24;
use alloy_primitives::{keccak256, Address, B256, U160};
use alloy_sol_types::SolValue;
use uniswap_sdk_core::prelude::*;
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
pub struct Pool<C0, C1, TP = NoTickDataProvider>
where
    C0: Currency,
    C1: Currency,
    TP: TickDataProvider,
{
    pub currency0: C0,
    pub currency1: C1,
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

impl<C0, C1> Pool<C0, C1>
where
    C0: Currency,
    C1: Currency,
{
    fn sort_currency(currency_a: &C0, currency_b: &C1) -> (Address, Address) {
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
        currency_a: &C0,
        currency_b: &C1,
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
        currency_a: &C0,
        currency_b: &C1,
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
}
