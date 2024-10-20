use derive_more::From;
use uniswap_sdk_core::error::Error as CoreError;
use uniswap_v3_sdk::error::Error as V3Error;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, From)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum Error {
    /// Thrown when an error occurs in the core library.
    #[cfg_attr(feature = "std", error("{0}"))]
    Core(#[cfg_attr(not(feature = "std"), from)] CoreError),

    /// Thrown when an error occurs in the v3 library.
    #[cfg_attr(feature = "std", error("{0}"))]
    V3(#[cfg_attr(not(feature = "std"), from)] V3Error),
}
