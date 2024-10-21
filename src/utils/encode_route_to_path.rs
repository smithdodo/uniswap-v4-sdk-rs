use crate::prelude::{Pool, Route};
use alloy_primitives::{aliases::U24, Address, Bytes};
use uniswap_sdk_core::prelude::*;
use uniswap_v3_sdk::prelude::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PathKey<I: TickIndex> {
    intermediate_currency: Address,
    fee: U24,
    tick_spacing: I,
    hooks: Address,
    hook_data: Bytes,
}

#[inline]
pub fn encode_route_to_path<TInput, TOutput, TP>(
    route: &Route<TInput, TOutput, TP>,
    exact_output: bool,
) -> Vec<PathKey<TP::Index>>
where
    TInput: BaseCurrency,
    TOutput: BaseCurrency,
    TP: TickDataProvider,
{
    let mut path_keys: Vec<PathKey<_>> = Vec::with_capacity(route.pools.len());
    if exact_output {
        let mut output_currency = &route.path_output;
        for pool in route.pools.iter().rev() {
            let (next_currency, key) = get_next_path_key(pool, output_currency);
            path_keys.push(key);
            output_currency = next_currency;
        }
        path_keys.reverse();
    } else {
        let mut input_currency = &route.path_input;
        for pool in &route.pools {
            let (next_currency, key) = get_next_path_key(pool, input_currency);
            path_keys.push(key);
            input_currency = next_currency;
        }
    }
    path_keys
}

#[inline]
fn get_next_path_key<'a, TInput, TP>(
    pool: &'a Pool<TP>,
    input_currency: &'a TInput,
) -> (&'a Currency, PathKey<TP::Index>)
where
    TInput: BaseCurrency,
    TP: TickDataProvider,
{
    let next_currency = if input_currency.equals(&pool.currency0) {
        &pool.currency1
    } else {
        &pool.currency0
    };
    (
        next_currency,
        PathKey {
            intermediate_currency: if next_currency.is_native() {
                Address::ZERO
            } else {
                next_currency.address()
            },
            fee: pool.fee,
            tick_spacing: pool.tick_spacing,
            hooks: pool.hooks,
            hook_data: Bytes::default(),
        },
    )
}
