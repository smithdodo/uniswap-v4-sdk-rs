use uniswap_sdk_core::prelude::*;

#[inline]
pub fn sorts_before(currency_a: &Currency, currency_b: &Currency) -> Result<bool, Error> {
    if currency_a.is_native() {
        return Ok(true);
    }
    if currency_b.is_native() {
        return Ok(false);
    }
    currency_a.wrapped().sorts_before(currency_b.wrapped())
}
