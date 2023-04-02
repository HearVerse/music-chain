#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

use frame_support::traits::Currency;
use sp_std::prelude::*;

pub use pallet::*;


pub use weights::WeightInfo;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
type AssetIdOf<T> = <T as Config>::AssetId;
type AssetBalanceOf<T> = <T as Config>::AssetBalance;
type ContractsBalanceOf<T> =
	<<T as pallet_contracts::Config>::Currency as frame_support::traits::Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use codec::EncodeLike;
	use frame_support::{
		inherent::Vec,
		pallet_prelude::*,
		sp_runtime::{
			traits::{
				AccountIdConversion, CheckedAdd, CheckedMul, CheckedSub, Convert, One, Saturating,
				Zero,
			},
			FixedPointNumber, FixedPointOperand, FixedU128,
		},
		traits::{
			fungibles::{Create, Destroy, Inspect, Mutate, Transfer},
			tokens::{AssetId, Balance, WithdrawConsequence},
			ExistenceRequirement, WithdrawReasons,
		},
		transactional, PalletId,
	};

	use frame_system::{pallet_prelude::*, Origin};
	use sp_std::fmt::Debug;
	pub const MAX_LENGTH: usize = 50;
	#[pallet::config]
	pub trait Config: frame_system::Config + TypeInfo + pallet_contracts::Config {
		/// Pallet ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency trait.
		type Currency: Currency<Self::AccountId>;

		/// The balance type for assets (i.e. tokens).
		type AssetBalance: Balance
			+ FixedPointOperand
			+ MaxEncodedLen
			+ MaybeSerializeDeserialize
			+ TypeInfo;

		// Two-way conversion between asset and currency balances
		type AssetToCurrencyBalance: Convert<Self::AssetBalance, BalanceOf<Self>>;
		type CurrencyToAssetBalance: Convert<BalanceOf<Self>, Self::AssetBalance>;

		/// The asset ID type.
		type AssetId: MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo
			+ Clone
			+ Debug
			+ PartialEq
			+ EncodeLike
			+ Decode;

		/// The type for tradable assets.
		type Assets: Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ Transfer<Self::AccountId>;

		/// The type for liquidity tokens.
		type AssetRegistry: Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ Mutate<Self::AccountId>
			+ Create<Self::AccountId>
			+ Destroy<Self::AccountId>;

		/// Information on runtime weights.
		type WeightInfo: WeightInfo;

		/// Provider fee numerator.
		#[pallet::constant]
		type ProviderFeeNumerator: Get<BalanceOf<Self>>;

		/// Provider fee denominator.
		#[pallet::constant]
		type ProviderFeeDenominator: Get<BalanceOf<Self>>;

		/// Minimum currency deposit for a new exchange.
		#[pallet::constant]
		type MinDeposit: Get<BalanceOf<Self>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new exchange was created [asset_id, liquidity_token_id]
		ExchangeCreated(AssetIdOf<T>, AssetIdOf<T>),
		/// Liquidity was added to an exchange [provider_id, asset_id, currency_amount,
		/// token_amount, liquidity_minted]
		LiquidityAdded(
			T::AccountId,
			AssetIdOf<T>,
			BalanceOf<T>,
			AssetBalanceOf<T>,
			AssetBalanceOf<T>,
		),
		/// Liquidity was removed from an exchange [provider_id, asset_id, currency_amount,
		/// token_amount, liquidity_amount]
		LiquidityRemoved(
			T::AccountId,
			AssetIdOf<T>,
			BalanceOf<T>,
			AssetBalanceOf<T>,
			AssetBalanceOf<T>,
		),
		/// Currency was traded for an asset [asset_id, buyer_id, recipient_id, currency_amount,
		/// token_amount]
		CurrencyTradedForAsset(
			AssetIdOf<T>,
			T::AccountId,
			T::AccountId,
			BalanceOf<T>,
			AssetBalanceOf<T>,
		),
		/// An asset was traded for currency [asset_id, buyer_id, recipient_id, currency_amount,
		/// token_amount]
		AssetTradedForCurrency(
			AssetIdOf<T>,
			T::AccountId,
			T::AccountId,
			BalanceOf<T>,
			AssetBalanceOf<T>,
		),
		/// Event to display when call is made from the extrinsic to a smart contract
		CalledContractFromPallet(T::AccountId),
		/// Event to display when call is made from a smart contract to the extrinsic
		CalledPalletFromContract(u32),
		TotalSupply(T::AccountId, BalanceOf<T>),

	}

	#[pallet::error]
	pub enum Error<T> {
		/// Asset with the specified ID does not exist
		AssetNotFound,
		/// Exchange for the given asset already exists
		ExchangeAlreadyExists,
		/// Provided liquidity token ID is already taken
		TokenIdTaken,
		/// Not enough free balance to add liquidity or perform trade
		BalanceTooLow,
		/// Not enough tokens to add liquidity or perform trade
		NotEnoughTokens,
		/// Specified account doesn't own enough liquidity in the exchange
		ProviderLiquidityTooLow,
		/// No exchange found for the given `asset_id`
		ExchangeNotFound,
		/// Zero value provided for trade amount parameter
		TradeAmountIsZero,
		/// Zero value provided for `token_amount` parameter
		TokenAmountIsZero,
		/// Zero value provided for `max_tokens` parameter
		MaxTokensIsZero,
		/// Zero value provided for `currency_amount` parameter
		CurrencyAmountIsZero,
		/// Value provided for `currency_amount` parameter is too high
		CurrencyAmountTooHigh,
		/// Value provided for `currency_amount` parameter is too low
		CurrencyAmountTooLow,
		/// Zero value provided for `min_liquidity` parameter
		MinLiquidityIsZero,
		/// Value provided for `max_tokens` parameter is too low
		MaxTokensTooLow,
		/// Value provided for `min_liquidity` parameter is too high
		MinLiquidityTooHigh,
		/// Zero value provided for `liquidity_amount` parameter
		LiquidityAmountIsZero,
		/// Zero value provided for `min_currency` parameter
		MinCurrencyIsZero,
		/// Zero value provided for `min_tokens` parameter
		MinTokensIsZero,
		/// Value provided for `min_currency` parameter is too high
		MinCurrencyTooHigh,
		/// Value provided for `min_tokens` parameter is too high
		MinTokensTooHigh,
		/// Value provided for `max_currency` parameter is too low
		MaxCurrencyTooLow,
		/// Value provided for `min_bought_tokens` parameter is too high
		MinBoughtTokensTooHigh,
		/// Value provided for `max_sold_tokens` parameter is too low
		MaxSoldTokensTooLow,
		/// There is not enough liquidity in the exchange to perform trade
		NotEnoughLiquidity,
		/// Overflow occurred
		Overflow,
		/// Underflow occurred
		Underflow,
		/// Deadline specified for the operation has passed
		DeadlinePassed,
		InputTooLarge,
		ContractNotFound,
	}

	pub trait ConfigHelper: Config {
		fn pallet_account() -> AccountIdOf<Self>;
		fn currency_to_asset(curr_balance: BalanceOf<Self>) -> AssetBalanceOf<Self>;
		fn asset_to_currency(asset_balance: AssetBalanceOf<Self>) -> BalanceOf<Self>;
		fn net_amount_numerator() -> BalanceOf<Self>;
	}

	impl<T: Config> ConfigHelper for T {
		#[inline(always)]
		fn pallet_account() -> AccountIdOf<Self> {
			Self::PalletId::get().into_account_truncating()
		}

		#[inline(always)]
		fn currency_to_asset(curr_balance: BalanceOf<Self>) -> AssetBalanceOf<Self> {
			Self::CurrencyToAssetBalance::convert(curr_balance)
		}

		#[inline(always)]
		fn asset_to_currency(asset_balance: AssetBalanceOf<Self>) -> BalanceOf<Self> {
			Self::AssetToCurrencyBalance::convert(asset_balance)
		}

		#[inline(always)]
		fn net_amount_numerator() -> BalanceOf<Self> {
			Self::ProviderFeeDenominator::get()
				.checked_sub(&Self::ProviderFeeNumerator::get())
				.expect("Provider fee shouldn't be greater than 100%")
		}
	}

	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, MaxEncodedLen, TypeInfo,
	)]
	pub struct Exchange<AssetId, Balance, AssetBalance> {
		pub asset_id: AssetId,
		pub currency_reserve: Balance,
		pub token_reserve: AssetBalance,
		pub liquidity_token_id: AssetId,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[codec(mel_bound())]
	pub struct ContractToken {
		pub contractAddress: Vec<u8>,
	}
	impl MaxEncodedLen for ContractToken {
		fn max_encoded_len() -> usize {
			100
		}
	}

	type ExchangeOf<T> = Exchange<AssetIdOf<T>, BalanceOf<T>, AssetBalanceOf<T>>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn exchanges)]
	pub(super) type Exchanges<T: Config> =
		StorageMap<_, Twox64Concat, AssetIdOf<T>, ExchangeOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tokenContracts)]
	pub(super) type ContractTokens<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, ContractToken, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// #[pallet::call_index(9)]
		// #[pallet::weight(10_000)]
		// pub fn total_supply(origin: OriginFor<T>, dest: T::AccountId) -> DispatchResult {
		// 	Self::total_supply_internal(origin, dest)
		// }

		#[pallet::call_index(9)]
		#[pallet::weight(10_000)]
		pub fn total_supply(
			origin: OriginFor<T>,
			dest: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
		
			let mut selector: crate::Vec<u8> = [0xdb, 0x63, 0x75, 0xa8].into();
		
			let gas_limit: Weight = T::BlockWeights::get().max_block;
		
			let value: BalanceOf<T> = Default::default();
			let contracts_value: ContractsBalanceOf<T> =
				<T as pallet_contracts::Config>::Currency::free_balance(&dest);
		
			// Check if the contract exists
			// let contract_info = pallet_contracts::Pallet::<T>::get_contract_info(&dest);
			// ensure!(contract_info.is_some(), "Contract not found");
		
			let mut data = Vec::new();
			data.append(&mut selector);
		
			log::info!("data: {:?} -- {:?} -- {:?}", contracts_value, gas_limit, data);

			let result = pallet_contracts::Pallet::<T>::bare_call(
				who,
				dest.clone(),
				contracts_value,
				gas_limit,
				None,
				data,
				false,
				pallet_contracts::Determinism::Deterministic,
			);
			log::info!("result: {:?}", result.result);

			if let Ok(contract_result) = result.result {
				let data = contract_result.data;
				let total_supply = BalanceOf::<T>::decode(&mut data.as_slice()).unwrap_or_default();
				Self::deposit_event(Event::TotalSupply(dest, total_supply));
				Ok(().into())
			} else {
				Ok(().into())
			}
		}
		
	}

	impl<T: Config> Pallet<T> {
		// fn total_supply_internal(origin: OriginFor<T>, dest: T::AccountId) -> DispatchResult {
		// 	let who = ensure_signed(origin)?;
		// 	ensure!(<ContractTokens<T>>::contains_key(&dest), Error::<T>::ContractNotFound);

		// 	let contract_token = ContractTokens::<T>::get(&dest).unwrap();
		// 	let contract_address =
		// 		T::AccountId::decode(&mut &contract_token.contractAddress[..]).unwrap();
		// 		let value = <T as pallet::Config>::Currency::free_balance(&contract_address);

		// 	// Convert value to u32 for the event
		// 	let contracts_value: u32 = value.saturated_into();

		// 	Self::deposit_event(Event::CalledPalletFromContract(contracts_value));
		// 	Ok(())
		// }
	}
}
