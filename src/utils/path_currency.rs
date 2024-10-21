use crate::prelude::{Error, Pool};
use uniswap_sdk_core::prelude::{BaseCurrency, Currency, CurrencyAmount};
use uniswap_v3_sdk::prelude::TickDataProvider;

#[inline]
pub fn amount_with_path_currency<TP: TickDataProvider>(
    amount: CurrencyAmount<Currency>,
    pool: &Pool<TP>,
) -> Result<CurrencyAmount<Currency>, Error> {
    Ok(CurrencyAmount::from_fractional_amount(
        get_path_currency(amount.meta.currency, pool)?,
        amount.numerator,
        amount.denominator,
    )?)
}

#[inline]
pub fn get_path_currency<TP: TickDataProvider>(
    currency: Currency,
    pool: &Pool<TP>,
) -> Result<Currency, Error> {
    if pool.involves_currency(&currency) {
        Ok(currency)
    } else if pool.involves_currency(currency.wrapped()) {
        Ok(Currency::Token(currency.wrapped().clone()))
    } else if pool.currency0.wrapped().equals(&currency) {
        Ok(pool.currency0.clone())
    } else if pool.currency1.wrapped().equals(&currency) {
        Ok(pool.currency1.clone())
    } else {
        Err(Error::InvalidCurrency)
    }
}
