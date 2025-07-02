# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust implementation of the Uniswap V4 SDK - a feature-complete port of the TypeScript V4 SDK. The library is designed to be `no_std` by default with optional standard library and extension features.

## Key Commands

### Build Commands
- `cargo build` - Build core library (no_std)
- `cargo build --features std` - Build with standard library support
- `cargo build --features extensions` - Build with extensions (requires network access)
- `cargo build --all-features` - Build with all features enabled

### Test Commands
- `cargo test` - Run core tests
- `cargo test --features std` - Test with std feature
- `cargo test --features extensions --lib extensions -- --test-threads=1` - Test extensions (single-threaded due to RPC calls)
- `cargo test --doc --all-features` - Run documentation tests

### Development Commands
- `cargo clippy --all-targets --all-features -- -D warnings` - Run linter (must pass with no warnings)
- `cargo fmt --all -- --check` - Check code formatting
- `cargo fmt` - Auto-format code

### Running a Single Test
```bash
cargo test test_name -- --exact
cargo test module::test_name -- --exact
```

## Architecture Overview

### Core Structure
The SDK follows a domain-driven design with clear separation of concerns:

- **Entities** (`src/entities/`): Core domain models
  - `Pool`: V4 pool representation with liquidity and pricing logic
  - `Position`: Liquidity position management
  - `Route`: Trading route definitions connecting multiple pools
  - `Trade`: Trade execution logic with slippage protection

- **Utilities** (`src/utils/`): Stateless helper functions
  - `v4_planner.rs`: Constructs V4 transaction calldata
  - `v4_position_planner.rs`: Position-specific transaction planning
  - `price_tick_conversions.rs`: Price/tick mathematical conversions
  - `encode_route_to_path.rs`: Route encoding for multi-hop trades

- **Extensions** (`src/extensions/`): Optional enhanced features (feature-gated)
  - `pool_manager_lens.rs`: Direct querying of pool manager state
  - `simple_tick_data_provider.rs`: RPC-based tick data fetching

### Key Design Patterns

1. **Feature Flags**: 
   - Core functionality is `no_std` compatible
   - `std` feature adds standard library support
   - `extensions` feature enables network-dependent functionality

2. **Type Safety**: 
   - Strong typing with `PoolKey`, `PathKey`, `Currency` types
   - Comprehensive error handling with custom error types
   - Builder pattern for complex operations (e.g., `MintPositionParams`)

3. **V3 SDK Integration**: 
   - Extends `uniswap-v3-sdk` types for V4-specific behavior
   - Reuses core mathematical functions and entity structures
   - Adds V4-specific features like hooks and dynamic fees

### Critical Implementation Details

- **Hook System**: V4 pools support hooks - check `hook.rs` for hook flag encoding
- **Dynamic Fees**: Unlike V3, V4 supports dynamic fee tiers
- **Currency Abstraction**: V4 supports both ERC20 tokens and native ETH as `Currency`
- **Action Encoding**: All operations use the `Actions` pattern for transaction building

## Environment Requirements

- **Rust Version**: MSRV 1.83 (verify with `rustc --version`)
- **For Extensions Testing**: Requires `MAINNET_RPC_URL` environment variable
- **Dependencies**: All managed through Cargo.toml - no external setup needed

## Testing Approach

- Unit tests are colocated with implementation files
- Integration tests use mock data from `src/tests.rs`
- Extension tests require mainnet RPC access and run single-threaded
- Always run tests before committing changes