use alloc::string::String;
use alloy_primitives::ChainId;
use uniswap_sdk_core::prelude::{BaseCurrency, Currency as CurrencyTrait, Ether, Token};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Currency {
    NativeCurrency(Ether),
    Token(Token),
}

impl BaseCurrency for Currency {
    fn is_native(&self) -> bool {
        matches!(self, Currency::NativeCurrency(_))
    }

    fn is_token(&self) -> bool {
        matches!(self, Currency::Token(_))
    }

    fn chain_id(&self) -> ChainId {
        match self {
            Currency::NativeCurrency(ether) => ether.chain_id(),
            Currency::Token(token) => token.chain_id(),
        }
    }

    fn decimals(&self) -> u8 {
        match self {
            Currency::NativeCurrency(ether) => ether.decimals(),
            Currency::Token(token) => token.decimals(),
        }
    }

    fn symbol(&self) -> Option<&String> {
        match self {
            Currency::NativeCurrency(ether) => ether.symbol(),
            Currency::Token(token) => token.symbol(),
        }
    }

    fn name(&self) -> Option<&String> {
        match self {
            Currency::NativeCurrency(ether) => ether.name(),
            Currency::Token(token) => token.name(),
        }
    }
}

impl CurrencyTrait for Currency {
    fn equals(&self, other: &impl CurrencyTrait) -> bool {
        match self {
            Currency::NativeCurrency(ether) => ether.equals(other),
            Currency::Token(token) => token.equals(other),
        }
    }

    fn wrapped(&self) -> &Token {
        match self {
            Currency::NativeCurrency(ether) => ether.wrapped(),
            Currency::Token(token) => token,
        }
    }
}
