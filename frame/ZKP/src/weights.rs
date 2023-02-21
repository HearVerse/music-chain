// Skip Rust code formatting, as this file contains macros.
#![cfg_attr(rustfmt, rustfmt_skip)]
// Allow unused parentheses in this file.
#![allow(unused_parens)]
// Allow unused imports in this file.
#![allow(unused_imports)]

// Import required types from the `frame_support` and `sp_std` crates.
use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

// Define the `WeightInfo` trait, which is used to calculate the weight of certain runtime functions.
pub trait WeightInfo {
    // Calculate the weight of a function that sets up a verification benchmark.
    fn setup_verification_benchmark(len:usize) -> Weight;

    // Calculate the weight of a function that verifies a benchmark.
    fn verify_benchmark(len:usize) -> Weight;
}

// Define the `SubstrateWeight` struct.
pub struct SubstrateWeight<T>(PhantomData<T>);

// Implement the `WeightInfo` trait for the `SubstrateWeight` struct.
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    // Calculate the weight of a function that sets up a verification benchmark.
    fn setup_verification_benchmark(len: usize) -> Weight {
        // Minimum execution time: 21_000 nanoseconds.
        // Calculate the weight cost of a function based on the number of input bytes.
        // Multiply the cost per byte by the number of bytes.
        // Add the weight cost of writing a value of `1` to the runtime storage.
        Weight::from_ref_time(22_000_000 as u64)
            .saturating_mul(len as u64)
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }

    // Calculate the weight of a function that verifies a benchmark.
    fn verify_benchmark(len: usize) -> Weight {
        // Minimum execution time: 31_000 nanoseconds.
        // Calculate the weight cost of a function based on the number of input bytes.
        // Multiply the cost per byte by the number of bytes.
        // Add the weight cost of reading a value of `1` from the runtime storage.
        // Add the weight cost of writing a value of `1` to the runtime storage.
        Weight::from_ref_time(32_000_000 as u64)
            .saturating_mul(len as u64)
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
}

// Implement the `WeightInfo` trait for the unit type.
impl WeightInfo for () {
    // Return a weight of `0` for a function that sets up a verification benchmark.
    fn setup_verification_benchmark(_len: usize) -> Weight {
        Weight::zero()
    }

    // Return a weight of `0` for a function that verifies a benchmark.
    fn verify_benchmark(_len: usize) -> Weight {
        Weight::zero()
    }
}
