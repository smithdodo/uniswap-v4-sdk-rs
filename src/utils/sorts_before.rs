use uniswap_sdk_core::prelude::*;

pub fn sorts_before(currency_a: &impl Currency, currency_b: &impl Currency) -> bool {
    if currency_a.is_native() {
        return true;
    }
    if currency_b.is_native() {
        return false;
    }
    currency_a
        .wrapped()
        .sorts_before(currency_b.wrapped())
        .unwrap()
}
