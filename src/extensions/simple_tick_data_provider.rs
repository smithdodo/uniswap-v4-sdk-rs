//! ## Simple Tick Data Provider
//! A data provider that fetches tick data from the Uniswap V4 pool manager contract on the fly
//! using [`PoolManagerLens`].

use crate::prelude::{map_contract_error, PoolManagerLens};
use alloy::{eips::BlockId, providers::DynProvider};
use alloy_primitives::{aliases::I24, Address, B256, U256};
use uniswap_v3_sdk::prelude::*;

#[derive(Clone, Debug)]
pub struct SimpleTickDataProvider<I = I24>
where
    I: TickIndex,
{
    pub lens: PoolManagerLens,
    pub pool_id: B256,
    pub block_id: Option<BlockId>,
    _tick_index: core::marker::PhantomData<I>,
}

impl<I> SimpleTickDataProvider<I>
where
    I: TickIndex,
{
    #[inline]
    pub fn new(
        manager: Address,
        pool_id: B256,
        provider: DynProvider,
        block_id: Option<BlockId>,
    ) -> Self {
        Self {
            lens: PoolManagerLens::new(manager, provider),
            pool_id,
            block_id,
            _tick_index: core::marker::PhantomData,
        }
    }

    #[inline]
    pub const fn block_id(mut self, block_id: Option<BlockId>) -> Self {
        self.block_id = block_id;
        self
    }

    #[inline]
    pub const fn pool_id(mut self, pool_id: B256) -> Self {
        self.pool_id = pool_id;
        self
    }
}

impl<I> TickBitMapProvider for SimpleTickDataProvider<I>
where
    I: TickIndex,
{
    type Index = I;

    #[inline]
    async fn get_word(&self, index: Self::Index) -> Result<U256, Error> {
        self.lens
            .get_tick_bitmap(self.pool_id, index, self.block_id)
            .await
            .map_err(map_contract_error)
    }
}

impl<I> TickDataProvider for SimpleTickDataProvider<I>
where
    I: TickIndex,
{
    type Index = I;

    #[inline]
    async fn get_tick(&self, index: Self::Index) -> Result<Tick<Self::Index>, Error> {
        let (liquidity_gross, liquidity_net) = self
            .lens
            .get_tick_liquidity(self.pool_id, index, self.block_id)
            .await
            .map_err(map_contract_error)?;
        Ok(Tick {
            index,
            liquidity_gross,
            liquidity_net,
        })
    }

    #[inline]
    async fn next_initialized_tick_within_one_word(
        &self,
        tick: Self::Index,
        lte: bool,
        tick_spacing: Self::Index,
    ) -> Result<(Self::Index, bool), Error> {
        TickBitMapProvider::next_initialized_tick_within_one_word(self, tick, lte, tick_spacing)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    use uniswap_sdk_core::addresses::CHAIN_TO_ADDRESSES_MAP;

    const TICK_SPACING: i32 = 10;

    #[tokio::test]
    async fn test_v4_simple_tick_data_provider() -> Result<(), Error> {
        let provider = super::SimpleTickDataProvider::new(
            CHAIN_TO_ADDRESSES_MAP
                .get(&1)
                .unwrap()
                .v4_pool_manager
                .unwrap(),
            *POOL_ID_ETH_USDC,
            PROVIDER.clone(),
            BLOCK_ID,
        );

        let slot0 = STATE_VIEW
            .getSlot0(*POOL_ID_ETH_USDC)
            .block(BLOCK_ID.unwrap())
            .call()
            .await?;

        // Find a populated tick based on the current state
        let word = slot0.tick.as_i32().compress(TICK_SPACING).position().0;

        // Get the bitmap at the current word
        let bitmap = provider.get_word(word).await?;
        assert_ne!(bitmap, U256::ZERO, "Bitmap should not be empty");

        // Find the first initialized tick in the bitmap
        let msb = most_significant_bit(bitmap);
        let tick = ((word << 8) + msb as i32) * TICK_SPACING;

        // Get the tick data and verify it's populated
        let tick_info = provider.get_tick(tick).await?;
        assert_eq!(tick_info.index, -202270);
        assert_eq!(tick_info.liquidity_gross, 847325330774525298);
        assert_eq!(tick_info.liquidity_net, -847325330774525298);

        // 1. Find next tick when going left (decreasing)
        let (found_tick, initialized) = TickDataProvider::next_initialized_tick_within_one_word(
            &provider,
            tick,
            true,
            TICK_SPACING,
        )
        .await?;
        // Should find our initialized tick
        assert_eq!(found_tick, tick);
        assert!(initialized, "Tick should be initialized");

        // 2. Finding the next initialized tick when going left (decreasing)
        let (found_tick, initialized) = TickDataProvider::next_initialized_tick_within_one_word(
            &provider,
            tick - TICK_SPACING,
            true,
            TICK_SPACING,
        )
        .await?;
        // Should find the next initialized tick
        assert_eq!(found_tick, -202300);
        assert!(initialized, "Tick should be initialized");

        // 3. Find the next tick when going right (increasing)
        let (found_tick, initialized) = TickDataProvider::next_initialized_tick_within_one_word(
            &provider,
            tick - TICK_SPACING,
            false,
            TICK_SPACING,
        )
        .await?;
        // Should find our initialized tick
        assert_eq!(found_tick, tick);
        assert!(initialized, "Tick should be initialized");

        // 4. Test at the edge of the tick range
        let (tick, initialized) = TickDataProvider::next_initialized_tick_within_one_word(
            &provider,
            MIN_TICK_I32 + TICK_SPACING,
            true,
            TICK_SPACING,
        )
        .await?;
        assert_eq!(tick, -887270);
        assert!(initialized);
        Ok(())
    }
}
