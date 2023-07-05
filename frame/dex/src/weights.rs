
#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_dex`.
pub trait WeightInfo {
	fn create_exchange() -> Weight;
	fn add_liquidity() -> Weight;
	fn remove_liquidity() -> Weight;
	fn currency_to_asset() -> Weight;
	fn asset_to_currency() -> Weight;
	fn asset_to_asset() -> Weight;

}

/// Weight functions for `pallet_dex`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Assets Asset (r:2 w:1)
	// Storage: Dex Exchanges (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Assets Account (r:3 w:3)
	fn create_exchange() -> Weight {
		Weight::from_ref_time(103_019_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	// Storage: Assets Asset (r:2 w:2)
	// Storage: Assets Account (r:3 w:3)
	// Storage: Dex Exchanges (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn add_liquidity() -> Weight {
		Weight::from_ref_time(89_032_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	// Storage: Dex Exchanges (r:1 w:1)
	// Storage: Assets Asset (r:2 w:2)
	// Storage: Assets Account (r:3 w:3)
	// Storage: System Account (r:1 w:1)
	fn remove_liquidity() -> Weight {
		Weight::from_ref_time(88_652_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	// Storage: Dex Exchanges (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	fn currency_to_asset() -> Weight {
		Weight::from_ref_time(70_294_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: Dex Exchanges (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn asset_to_currency() -> Weight {
		Weight::from_ref_time(72_349_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: Dex Exchanges (r:2 w:2)
	// Storage: Assets Asset (r:2 w:2)
	// Storage: Assets Account (r:4 w:4)
	fn asset_to_asset() -> Weight {
		Weight::from_ref_time(99_152_000)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(8))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Assets Asset (r:2 w:1)
	// Storage: Dex Exchanges (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Assets Account (r:3 w:3)
	fn create_exchange() -> Weight {
		Weight::from_ref_time(103_019_000)
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(7))
	}
	// Storage: Assets Asset (r:2 w:2)
	// Storage: Assets Account (r:3 w:3)
	// Storage: Dex Exchanges (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn add_liquidity() -> Weight {
		Weight::from_ref_time(89_032_000)
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(7))
	}
	// Storage: Dex Exchanges (r:1 w:1)
	// Storage: Assets Asset (r:2 w:2)
	// Storage: Assets Account (r:3 w:3)
	// Storage: System Account (r:1 w:1)
	fn remove_liquidity() -> Weight {
		Weight::from_ref_time(88_652_000)
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(7))
	}
	// Storage: Dex Exchanges (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	fn currency_to_asset() -> Weight {
		Weight::from_ref_time(70_294_000)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(5))
	}
	// Storage: Dex Exchanges (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn asset_to_currency() -> Weight {
		Weight::from_ref_time(72_349_000)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(5))
	}
	// Storage: Dex Exchanges (r:2 w:2)
	// Storage: Assets Asset (r:2 w:2)
	// Storage: Assets Account (r:4 w:4)
	fn asset_to_asset() -> Weight {
		Weight::from_ref_time(99_152_000)
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(8))
	}
}