#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

pub mod verifier;
pub use verifier::*;

pub mod weights;
pub use weights::*;

use frame_support::storage::bounded_vec::BoundedVec;

type ProofDef<T> = BoundedVec<u8, <T as Config>::MaxProofLength>;
type VerificationKey<T> = BoundedVec<u8, <T as Config>::MaxVerificationKeyLength>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;

		/// The maximum length of the proof.
		#[pallet::constant]
		type MaxProofLength: Get<u32>;

		/// The maximum length of the verification key.
		#[pallet::constant]
		type MaxVerificationKeyLength: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		VerificationSetupCompleted,
		VerificationProofSet,
		VerificationSuccess,
		VerificationFailed,
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The verification key is to long.
		TooLongVerificationKey,
		/// The proof is too long.
		TooLongProof,
		/// The proof is too short.
		ProofIsEmpty,
		/// Verification key, not set.
		VerificationKeyIsNotSet,
	}

	/// Storing a public input.
	#[pallet::storage]
	pub type PublicInputStorage<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Storing a proof.
	#[pallet::storage]
	pub type ProofStorage<T: Config> = StorageValue<_, ProofDef<T>, ValueQuery>;

	/// Storing a verification key.
	#[pallet::storage]
	pub type VerificationKeyStorage<T: Config> = StorageValue<_, VerificationKey<T>, ValueQuery>;

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::setup_verification_benchmark(vec_vk.len()))]
		pub fn setup_verification(
			_origin: OriginFor<T>,
			pub_input: u32,
			vec_vk: Vec<u8>,
		) -> DispatchResult {
			// Setting the public input data.
			PublicInputStorage::<T>::put(pub_input);

			// Setting the verification key.
			if vec_vk.is_empty() {
				VerificationKeyStorage::<T>::kill();
			} else {
				let vk: VerificationKey<T> =
					vec_vk.try_into().map_err(|_| Error::<T>::TooLongVerificationKey)?;

				VerificationKeyStorage::<T>::put(vk);
				Self::deposit_event(Event::<T>::VerificationSetupCompleted);
			}
			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::verify_benchmark(vec_proof.len()))]
		pub fn verify(_origin: OriginFor<T>, vec_proof: Vec<u8>) -> DispatchResult {
			
			ensure!(!vec_proof.is_empty(), Error::<T>::ProofIsEmpty);

			let proof: ProofDef<T> = vec_proof.try_into().map_err(|_| Error::<T>::TooLongProof)?;

			ProofStorage::<T>::put(proof.clone());

			let v = Verifier { key: <VerificationKeyStorage<T>>::get().clone().into_inner() };

			let public_input = PublicInputStorage::<T>::get().clone();

			let is_verify = v
				.verifier_proof(public_input, proof.into_inner())
				.map_err(|_| Error::<T>::VerificationKeyIsNotSet)?;

			if is_verify {
				Self::deposit_event(Event::<T>::VerificationSuccess);
			} else {
				Self::deposit_event(Event::<T>::VerificationFailed);
			}
			Ok(())
		}
	}
}
