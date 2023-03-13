#![cfg_attr(not(feature = "std"), no_std)]


use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	dispatch::{
		DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo, GetDispatchInfo,
		PostDispatchInfo,
	},
	ensure,
	traits::{Currency, Get, ReservableCurrency},
	weights::Weight,
	BoundedVec, RuntimeDebug,
};
use frame_system::{self as system, RawOrigin};
use scale_info::TypeInfo;
use sp_io::hashing::blake2_256;
use sp_runtime::{
	traits::{Dispatchable, TrailingZeroInput, Zero},
	DispatchError,
};
use sp_std::prelude::*;

pub mod weights;
pub use weights::WeightInfo;
pub use pallet::*;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The overarching call type.
		type RuntimeCall: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The base amount of currency needed to reserve for creating a multisig execution or to
		/// store a dispatch call for later.
		///
		/// This is held for an additional storage item whose value size is
		/// `4 + sizeof((BlockNumber, Balance, AccountId))` bytes and whose key size is
		/// `32 + sizeof(AccountId)` bytes.
		#[pallet::constant]
		type DepositBase: Get<BalanceOf<Self>>;

		/// The amount of currency needed per unit threshold when creating a multisig execution.
		///
		/// This is held for adding 32 bytes more into a pre-existing storage value.
		#[pallet::constant]
		type DepositFactor: Get<BalanceOf<Self>>;

		/// The maximum amount of signatories allowed in the multisig.
		#[pallet::constant]
		type MaxSignatories: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		VerificationSetupCompleted,
		VerificationProofSet,
		VerificationSuccess { who: T::AccountId },
		VerificationFailed,
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Public inputs mismatch
		PublicInputsMismatch,
		/// Public inputs vector is to long.
		TooLongPublicInputs,
		/// The verification key is to long.
		TooLongVerificationKey,
		/// The proof is too long.
		TooLongProof,
		/// The proof is too short.
		ProofIsEmpty,
		/// Verification key, not set.
		VerificationKeyIsNotSet,
		/// Malformed key
		MalformedVerificationKey,
		/// Malformed proof
		MalformedProof,
		/// Malformed public inputs
		MalformedPublicInputs,
		/// Curve is not supported
		NotSupportedCurve,
		/// Protocol is not supported
		NotSupportedProtocol,
		/// There was error during proof verification
		ProofVerificationError,
		/// Proof creation error
		ProofCreationError,
		/// Verification Key creation error
		VerificationKeyCreationError,
	}

	
	#[pallet::call]
	impl<T: Config> Pallet<T> {
	
	}


}