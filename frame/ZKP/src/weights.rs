#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
    fn setup_verification_benchmark(len:usize) -> Weight;
    fn verify_benchmark(len:usize) -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {

    fn setup_verification_benchmark(len: usize,) -> Weight {
		// Minimum execution time: 21_000 nanoseconds.
		Weight::from_ref_time(22_000_000 as u64).saturating_mul(len as u64)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}

    fn verify_benchmark(len: usize,) -> Weight {
		// Minimum execution time: 31_000 nanoseconds.
		Weight::from_ref_time(32_000_000 as u64).saturating_mul(len as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
}

impl WeightInfo for () {
    fn setup_verification_benchmark(_len: usize,) -> Weight {
        Weight::zero()
    }

	fn verify_benchmark(_len: usize,) -> Weight {
        Weight::zero()
    }
}