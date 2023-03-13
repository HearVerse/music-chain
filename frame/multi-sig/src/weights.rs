

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_multisig.
pub trait WeightInfo {
	fn as_multi_threshold_1(z: u32, ) -> Weight;
	fn as_multi_create(s: u32, z: u32, ) -> Weight;
	fn as_multi_approve(s: u32, z: u32, ) -> Weight;
	fn as_multi_complete(s: u32, z: u32, ) -> Weight;
	fn approve_as_multi_create(s: u32, ) -> Weight;
	fn approve_as_multi_approve(s: u32, ) -> Weight;
	fn cancel_as_multi(s: u32, ) -> Weight;
}

/// Weights for pallet_multisig using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    
	fn as_multi_threshold_1(z: u32, ) -> Weight {
        
		Weight::from_parts(12_464_828, 0)
        
			.saturating_add(Weight::from_parts(494, 0).saturating_mul(z.into()))
	}
    
	fn as_multi_create(s: u32, z: u32, ) -> Weight {
        
		Weight::from_parts(29_088_956, 5821)
        
			.saturating_add(Weight::from_parts(67_846, 0).saturating_mul(s.into()))
	
			.saturating_add(Weight::from_parts(1_523, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
    
	fn as_multi_approve(s: u32, z: u32, ) -> Weight {
        
		Weight::from_parts(20_479_380, 5821)
	
			.saturating_add(Weight::from_parts(64_116, 0).saturating_mul(s.into()))
		
			.saturating_add(Weight::from_parts(1_520, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	
    
	fn as_multi_complete(s: u32, z: u32, ) -> Weight {
        
		Weight::from_parts(32_311_615, 8424)

			.saturating_add(Weight::from_parts(85_999, 0).saturating_mul(s.into()))
	
			.saturating_add(Weight::from_parts(1_534, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
    
	fn approve_as_multi_create(s: u32, ) -> Weight {
        
		Weight::from_parts(27_802_216, 5821)
			.saturating_add(Weight::from_parts(69_282, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
    
	fn approve_as_multi_approve(s: u32, ) -> Weight {
        
		Weight::from_parts(19_095_404, 5821)
			.saturating_add(Weight::from_parts(66_914, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
    
	fn cancel_as_multi(s: u32, ) -> Weight {
        
		Weight::from_parts(28_702_686, 5821)
			.saturating_add(Weight::from_parts(69_419, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
    
	fn as_multi_threshold_1(z: u32, ) -> Weight {
        
		Weight::from_parts(12_464_828, 0)
			.saturating_add(Weight::from_parts(494, 0).saturating_mul(z.into()))
	}


	fn as_multi_create(s: u32, z: u32, ) -> Weight {
        
		Weight::from_parts(29_088_956, 5821)
			.saturating_add(Weight::from_parts(67_846, 0).saturating_mul(s.into()))
	
			.saturating_add(Weight::from_parts(1_523, 0).saturating_mul(z.into()))
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	fn as_multi_approve(s: u32, z: u32, ) -> Weight {
        
		Weight::from_parts(20_479_380, 5821)
			.saturating_add(Weight::from_parts(64_116, 0).saturating_mul(s.into()))

			.saturating_add(Weight::from_parts(1_520, 0).saturating_mul(z.into()))
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}


	fn as_multi_complete(s: u32, z: u32, ) -> Weight {
        
		Weight::from_parts(32_311_615, 8424)
			.saturating_add(Weight::from_parts(85_999, 0).saturating_mul(s.into()))

			.saturating_add(Weight::from_parts(1_534, 0).saturating_mul(z.into()))
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
    
	fn approve_as_multi_create(s: u32, ) -> Weight {
        
		Weight::from_parts(27_802_216, 5821)
			.saturating_add(Weight::from_parts(69_282, 0).saturating_mul(s.into()))
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	fn approve_as_multi_approve(s: u32, ) -> Weight {

		Weight::from_parts(19_095_404, 5821)
			.saturating_add(Weight::from_parts(66_914, 0).saturating_mul(s.into()))
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	fn cancel_as_multi(s: u32, ) -> Weight {
        
		Weight::from_parts(28_702_686, 5821)
			.saturating_add(Weight::from_parts(69_419, 0).saturating_mul(s.into()))
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}