#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;
use frame_support::{traits::Currency, Parameter};
pub use pallet::*;
use sp_runtime::{traits::AccountIdConversion, MultiAddress};
use sp_std::prelude::*;
pub use weights::WeightInfo;
// use pallet_contracts::ExecReturnValue;
use pallet_contracts_primitives::{ExecReturnValue, ReturnFlags};
use scale_info::prelude::string::String;

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
				AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedMul, CheckedSub,
				Convert, Member, One, Saturating, Zero,
			},
			DispatchResult, FixedPointNumber, FixedPointOperand, FixedU128,
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
		TokenTransferred(T::AccountId, T::AccountId, BalanceOf<T>),
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
		TokenTransferFailed,
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

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub struct Exchange<T: Config> {
		pub asset_id: T::AccountId,
		pub currency_reserve: BalanceOf<T>,
		pub token_reserve: AssetBalanceOf<T>,
		pub liquidity_token_id: T::AccountId,
	}

	type ExchangeOf<T> = Exchange<T>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn exchanges)]
	pub(super) type Exchanges<T: Config> =
		StorageMap<_, Twox64Concat, AssetIdOf<T>, ExchangeOf<T>, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::create_exchange())]
		#[transactional]
		pub fn create_exchange(
			origin: OriginFor<T>,
			asset_id: T::AccountId,
			liquidity_token_id: T::AccountId,
			currency_amount: BalanceOf<T>,
			token_amount: BalanceOf<T>,
		) -> DispatchResult {
			// -------------------------- Validation part --------------------------
			let caller = ensure_signed(origin)?;
			ensure!(currency_amount >= T::MinDeposit::get(), Error::<T>::CurrencyAmountTooLow);
			ensure!(token_amount > Zero::zero(), Error::<T>::TokenAmountIsZero);

			// -------------------------- Update storage ---------------------------
			let exchange = Exchange {
				asset_id: asset_id.clone(),
				currency_reserve: <BalanceOf<T>>::zero(),
				token_reserve: <AssetBalanceOf<T>>::zero(),
				liquidity_token_id: liquidity_token_id.clone(),
			};

			let liquidity_minted = T::currency_to_asset(currency_amount);
			
			Self::do_add_liquidity(
				exchange,
				currency_amount,
				token_amount,
				liquidity_minted,
				caller,
			)?;

			// ---------------------------- Emit event -----------------------------
			// Self::deposit_event(Event::ExchangeCreated(asset_id, liquidity_token_id));
			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(10_000)]
		pub fn transfer_token(
			origin: OriginFor<T>,
			contract_address: T::AccountId,
			to: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let transfer = Self::transfer_token_from_owner(&sender, contract_address, to, amount);

			match transfer {
				Ok(()) => Ok(().into()),
				Err(e) => Err(e),
			}
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn transfer_token_from_owner(
			origin: &T::AccountId,
			contract_address: T::AccountId,
			to: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let method_id: [u8; 4] = [0x84, 0xa1, 0x5d, 0xa1];

			let gas_limit: Weight = T::BlockWeights::get().max_block;

			let contracts_value: ContractsBalanceOf<T> =
				<T as pallet_contracts::Config>::Currency::free_balance(&contract_address);

			let mut data = method_id.to_vec();
			data.extend(to.encode());
			data.extend(amount.encode());

			let result = pallet_contracts::Pallet::<T>::bare_call(
				origin.clone(),
				contract_address.clone(),
				contracts_value,
				gas_limit,
				None,
				data,
				false,
				pallet_contracts::Determinism::Deterministic,
			);
			log::info!("result: {:?}", result.result);

			if let Ok(contract_result) = result.result {
				// Check if the contract call was successful
				if !contract_result.flags.contains(pallet_contracts_primitives::ReturnFlags::REVERT)
				{
					Self::deposit_event(Event::TokenTransferred(contract_address, to, amount));
					Ok(().into())
				} else {
					Err(Error::<T>::TokenTransferFailed)?
				}
			} else {
				Err(Error::<T>::TokenTransferFailed)?
			}
		}

		#[transactional]
		fn do_add_liquidity(
			mut exchange: ExchangeOf<T>,
			currency_amount: BalanceOf<T>,
			token_amount: BalanceOf<T>,
			liquidity_minted: AssetBalanceOf<T>,
			provider: AccountIdOf<T>,
		) -> DispatchResult {

			// --------------------- Currency & token transfer ---------------------

			let asset_id = exchange.asset_id;
			let pallet_account = T::pallet_account();
			log::info!("result: {:?}", pallet_account);
			// <T as pallet::Config>::Currency::transfer(
			// 	&provider,
			// 	&pallet_account,
			// 	currency_amount,
			// 	ExistenceRequirement::KeepAlive,
			// )?;

			// T::Assets::transfer(asset_id.clone(), &provider, &pallet_account, token_amount,
			// true)?; 
			
			// T::AssetRegistry::mint_into(
			// 	exchange.liquidity_token_id.clone(),
			// 	&provider,
			// 	liquidity_minted,
			// )?;

			// -------------------------- Balances update --------------------------
			// exchange.currency_reserve.saturating_accrue(currency_amount);
			// exchange.token_reserve.saturating_accrue(token_amount);
			// <Exchanges<T>>::insert(asset_id.clone(), exchange);

			// ---------------------------- Emit event -----------------------------
			// Self::deposit_event(Event::LiquidityAdded(
			// 	provider,
			// 	asset_id,
			// 	currency_amount,
			// 	token_amount,
			// 	liquidity_minted,
			// ));
			Ok(())
		}
	}
}
