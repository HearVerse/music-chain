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
// (sold_token_amount, currency_amount, bought_token_amount)
type AssetToAssetPrice<T> = (BalanceOf<T>, BalanceOf<T>);

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
		storage::transactional,
		traits::{
			fungibles::{Create, Destroy, Inspect, Mutate, Transfer},
			tokens::{AssetId, Balance, WithdrawConsequence},
			ExistenceRequirement, WithdrawReasons,
		},
		transactional, PalletId,
	};

	use frame_system::{ensure_signed, pallet_prelude::*, Origin};
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
			AccountIdOf<T>,
			AccountIdOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
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
		AssetToAssetPriceCalculated(AccountIdOf<T>, BalanceOf<T>, BalanceOf<T>),

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
		InsufficientOutputAmount,
		ExchangeDoesNotExist,
		ArithmeticUnderflow,
		ArithmeticOverflow,
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
		ZeroReserve,
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
		UnableToFetchTotalLiquidity,
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
		pub token_a: T::AccountId,
		pub token_a_reserve: BalanceOf<T>,
		pub token_b_reserve: BalanceOf<T>,
		pub token_b: T::AccountId,
		pub fee_numerator: BalanceOf<T>,
		pub fee_denominator: BalanceOf<T>,
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum TradeAmount<InputBalance, OutputBalance> {
		FixedInput { input_amount: InputBalance, min_output: OutputBalance },
		FixedOutput { max_input: InputBalance, output_amount: OutputBalance },
	}

	type ExchangeOf<T> = Exchange<T>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn exchanges)]
	pub(super) type Exchanges<T: Config> =
		StorageMap<_, Twox64Concat, (AccountIdOf<T>, AccountIdOf<T>), Exchange<T>, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::create_exchange())]
		#[transactional]
		pub fn create_exchange(
			origin: OriginFor<T>,
			token_a: AccountIdOf<T>,
			token_b: AccountIdOf<T>,
			currency_amount: BalanceOf<T>,
			token_amount: BalanceOf<T>,
			fee_numerator: BalanceOf<T>,
			fee_denominator: BalanceOf<T>,
		) -> DispatchResult {
			// -------------------------- Validation part --------------------------
			let caller = ensure_signed(origin)?;
			ensure!(currency_amount >= T::MinDeposit::get(), Error::<T>::CurrencyAmountTooLow);
			ensure!(token_amount > Zero::zero(), Error::<T>::TokenAmountIsZero);

			// -------------------------- Update storage ---------------------------
			let exchange = Exchange {
				token_a: token_a.clone(),
				token_a_reserve: <BalanceOf<T>>::zero(),
				token_b_reserve: <BalanceOf<T>>::zero(),
				token_b: token_b.clone(),
				fee_numerator,
				fee_denominator,
			};

			let liquidity_minted = T::currency_to_asset(currency_amount);
			log::info!("liquidity_minted: {:?}", liquidity_minted);

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

		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::create_exchange())]
		#[transactional]
		pub fn swap(
			origin: OriginFor<T>,
			token_a: AccountIdOf<T>,
			token_b: AccountIdOf<T>,
			input_amount: BalanceOf<T>,
			min_output: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let pallet_account = T::pallet_account();

			// Get the exchange info from storage
			let exchange = Self::exchanges((token_a.clone(), token_b.clone()))
				.ok_or(Error::<T>::ExchangeDoesNotExist)?;

			// Calculate the output amount
			let output_amount = Self::get_output_amount(
				&input_amount,
				&exchange.token_a_reserve,
				&exchange.token_b_reserve,
				&exchange.fee_numerator,
				&exchange.fee_denominator,
			)?;

			// Check if the output amount is greater than or equal to the minimum output
			ensure!(output_amount >= min_output, Error::<T>::InsufficientOutputAmount);

			Self::transfer_token_from_owner(
				&token_a,
				sender.clone(),
				pallet_account.clone(),
				input_amount,
			)?;

			Self::transfer_token_from_owner(
				&token_b,
				pallet_account.clone(),
				sender.clone(),
				output_amount,
			)?;

			// Update the reserves
			let updated_token_a = exchange
				.token_a_reserve
				.checked_add(&input_amount)
				.ok_or(Error::<T>::Overflow)?;

			let updated_token_b = exchange
				.token_b_reserve
				.checked_sub(&output_amount)
				.ok_or(Error::<T>::Underflow)?;

			Exchanges::<T>::insert(
				(token_a, token_b),
				Exchange {
					token_a_reserve: updated_token_a,
					token_b_reserve: updated_token_b,
					..exchange
				},
			);

			Ok(().into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::create_exchange())]
		#[transactional]
		pub fn asset_to_asset(
			origin: OriginFor<T>,
			sold_token_a: AccountIdOf<T>,
			sold_token_b: AccountIdOf<T>,
			bought_token_a: AccountIdOf<T>,
			bought_token_b: AccountIdOf<T>,
			amount: TradeAmount<BalanceOf<T>, BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
	
			// Fetching the exchanges
			let sold_asset_exchange = Self::exchanges((sold_token_a.clone(), sold_token_b.clone()))
				.ok_or(Error::<T>::ExchangeDoesNotExist)?;
	
			let bought_asset_exchange = Self::exchanges((bought_token_a.clone(), bought_token_b.clone()))
				.ok_or(Error::<T>::ExchangeDoesNotExist)?;
	
			// Verifying the trade amount
			Self::check_trade_amount(&amount)?;
	
			// Fetching the sold and bought token amounts
			let (sold_token_amount, bought_token_amount) = Self::get_asset_to_asset_price(
				&sold_asset_exchange,
				&bought_asset_exchange,
				amount,
			)?;
	
			// Implementing the trade
			// You need to ensure that the origin account has enough balance of the sold token
			// Also, you need to transfer the sold tokens from the origin account to the exchange
			// And transfer the bought tokens from the exchange to the origin account
			// After these operations, the reserve of the sold tokens will increase in the sold_asset_exchange
			// And the reserve of the bought tokens will decrease in the bought_asset_exchange
	
			// Updating the reserves
			let updated_sold_exchange = Self::update_reserve_after_sell(&sold_asset_exchange, sold_token_amount, sold_token_a == sold_asset_exchange.token_a)?;
			let updated_bought_exchange = Self::update_reserve_after_buy(&bought_asset_exchange, bought_token_amount, bought_token_a == bought_asset_exchange.token_a)?;
	
			// Updating the exchanges in the storage
			Self::update_exchange_storage((sold_token_a.clone(), sold_token_b.clone()), updated_sold_exchange);
			Self::update_exchange_storage((bought_token_a.clone(), bought_token_b.clone()), updated_bought_exchange);
	
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn get_exchange(asset_id: &AccountIdOf<T>) -> Result<ExchangeOf<T>, Error<T>> {
			<Exchanges<T>>::get((asset_id.clone(), asset_id.clone()))
				.ok_or(Error::<T>::ExchangeNotFound)
		}

		fn check_deadline(deadline: &T::BlockNumber) -> Result<(), Error<T>> {
			ensure!(deadline >= &<frame_system::Pallet<T>>::block_number(), Error::DeadlinePassed);
			Ok(())
		}

		pub fn transfer_token_from_owner(
			origin: &AccountIdOf<T>,
			contract_address: AccountIdOf<T>,
			to: AccountIdOf<T>,
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

		fn get_total_liquidity(token_id: &AccountIdOf<T>) -> Result<BalanceOf<T>, Error<T>> {
			let method_id: [u8; 4] = [0x18, 0x16, 0x1d, 0x71]; // This is the method ID for the ERC-20 totalSupply function

			let gas_limit: Weight = T::BlockWeights::get().max_block;

			let data = method_id.to_vec();

			let result = pallet_contracts::Pallet::<T>::bare_call(
				token_id.clone(),
				token_id.clone(),
				Zero::zero(),
				gas_limit,
				None,
				data,
				false,
				pallet_contracts::Determinism::Deterministic,
			);

			if let Ok(contract_result) = result.result {
				if !contract_result.flags.contains(pallet_contracts_primitives::ReturnFlags::REVERT)
				{
					let total_liquidity: BalanceOf<T> =
						Decode::decode(&mut &contract_result.data[..]).unwrap_or(Zero::zero());
					Ok(total_liquidity)
				} else {
					Err(Error::<T>::UnableToFetchTotalLiquidity)
				}
			} else {
				Err(Error::<T>::UnableToFetchTotalLiquidity)
			}
		}

		fn check_enough_currency(
			owner: &AccountIdOf<T>,
			token_id: &AccountIdOf<T>,
			required_amount: BalanceOf<T>,
		) -> Result<(), Error<T>> {
			let method_id: [u8; 4] = [0x70, 0xa0, 0x8a, 0x31]; // This is the method ID for the ERC-20 balanceOf function

			let gas_limit: Weight = T::BlockWeights::get().max_block;

			let mut data = method_id.to_vec();
			data.extend(owner.encode());

			let result = pallet_contracts::Pallet::<T>::bare_call(
				owner.clone(),
				token_id.clone(),
				Zero::zero(),
				gas_limit,
				None,
				data,
				false,
				pallet_contracts::Determinism::Deterministic,
			);

			if let Ok(contract_result) = result.result {
				if !contract_result.flags.contains(pallet_contracts_primitives::ReturnFlags::REVERT)
				{
					let balance: BalanceOf<T> =
						Decode::decode(&mut &contract_result.data[..]).unwrap_or(Zero::zero());
					balance >= required_amount;
					Ok(())
				} else {
					Err(Error::<T>::BalanceTooLow)
				}
			} else {
				Err(Error::<T>::BalanceTooLow)
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

			let asset_id = exchange.token_a.clone(); // Clone asset_id
			let liquidity_token_id = exchange.token_b.clone(); // Clone liquidity_token_id

			let pallet_account = T::pallet_account();
			log::info!("result: {:?}", pallet_account);

			let transfer_from_user_to_pallet = Self::transfer_token_from_owner(
				&provider,
				liquidity_token_id.clone(),
				pallet_account.clone(),
				token_amount,
			);

			let trnaasfer_from_pallet_to_user = Self::transfer_token_from_owner(
				&provider,
				asset_id.clone(),
				pallet_account.clone(),
				token_amount,
			);

			// -------------------------- Balances update --------------------------

			exchange.token_a_reserve.saturating_accrue(currency_amount);
			exchange.token_b_reserve.saturating_accrue(token_amount);

			Exchanges::<T>::insert((asset_id.clone(), liquidity_token_id.clone()), exchange);

			// ---------------------------- Emit event -----------------------------
			Self::deposit_event(Event::LiquidityAdded(
				provider,
				asset_id,
				currency_amount,
				token_amount,
				liquidity_minted,
			));
			Ok(())
		}

		pub(crate) fn get_output_amount(
			input_amount: &BalanceOf<T>,
			input_reserve: &BalanceOf<T>,
			output_reserve: &BalanceOf<T>,
			fee_numerator: &BalanceOf<T>,
			fee_denominator: &BalanceOf<T>,
		) -> Result<BalanceOf<T>, Error<T>> {
			debug_assert!(!input_reserve.is_zero());
			debug_assert!(!output_reserve.is_zero());

			// input_amount_with_fee = input_amount * (fee_denominator - fee_numerator)
			let input_amount_with_fee = input_amount
				.checked_mul(&fee_denominator.checked_sub(fee_numerator).ok_or(Error::Overflow)?)
				.ok_or(Error::Overflow)?;

			// numerator = input_amount_with_fee * output_reserve
			let numerator =
				input_amount_with_fee.checked_mul(output_reserve).ok_or(Error::Overflow)?;

			// denominator = input_reserve * fee_denominator + input_amount_with_fee
			let denominator = input_reserve
				.checked_mul(fee_denominator)
				.ok_or(Error::Overflow)?
				.checked_add(&input_amount_with_fee)
				.ok_or(Error::Overflow)?;

			Ok(numerator / denominator)
		}

		pub(crate) fn get_input_amount(
			output_amount: &BalanceOf<T>,
			input_reserve: &BalanceOf<T>,
			output_reserve: &BalanceOf<T>,
			fee_numerator: &BalanceOf<T>,
			fee_denominator: &BalanceOf<T>,
		) -> Result<BalanceOf<T>, Error<T>> {
			debug_assert!(!input_reserve.is_zero());
			debug_assert!(!output_reserve.is_zero());
			ensure!(*output_amount < *output_reserve, Error::<T>::NotEnoughLiquidity);

			// numerator = input_reserve * output_amount * fee_denominator
			let numerator = input_reserve
				.checked_mul(&output_amount)
				.ok_or(Error::Overflow)?
				.checked_mul(&fee_denominator)
				.ok_or(Error::Overflow)?;

			let denominator = output_reserve
				.checked_sub(output_amount)
				.ok_or(Error::Overflow)?
				.checked_mul(&fee_denominator.checked_sub(fee_numerator).ok_or(Error::Overflow)?)
				.ok_or(Error::Overflow)?;

			// (numerator / denominator) + 1
			Ok((numerator / denominator).saturating_add(BalanceOf::<T>::one()))
		}

		pub fn get_asset_to_asset_price(
			sold_asset_exchange: &Exchange<T>,
			bought_asset_exchange: &Exchange<T>,
			amount: TradeAmount<BalanceOf<T>, BalanceOf<T>>,
		) -> Result<AssetToAssetPrice<T>, Error<T>> {
			match amount {
				TradeAmount::FixedInput {
					input_amount: sold_token_amount,
					min_output: min_bought_tokens,
				} => {
					log::info!(
						"get_asset_to_asset_price: sold_token_amount: {:?}, min_bought_tokens: {:?}",
						sold_token_amount,
						min_bought_tokens);

					let currency_amount = Self::get_output_amount(
						&sold_token_amount,
						&sold_asset_exchange.token_a_reserve,
						&sold_asset_exchange.token_b_reserve,
						&sold_asset_exchange.fee_numerator,
						&sold_asset_exchange.fee_denominator,
					)?;
					let bought_token_amount = Self::get_output_amount(
						&currency_amount,
						&bought_asset_exchange.token_a_reserve,
						&bought_asset_exchange.token_b_reserve,
						&bought_asset_exchange.fee_numerator,
						&bought_asset_exchange.fee_denominator,
					)?;
					log::info!(
						"get_asset_to_asset_price: sold_token_amount: {:?}, bought_token_amount: {:?}",
						sold_token_amount,
						bought_token_amount);

					ensure!(
						bought_token_amount >= min_bought_tokens,
						Error::<T>::MinBoughtTokensTooHigh
					);
					Ok((
						sold_token_amount,   // should be a Balance
						bought_token_amount, // should be a Balance
					))
				},
				TradeAmount::FixedOutput {
					max_input: max_sold_tokens,
					output_amount: bought_token_amount,
				} => {
					let currency_amount = Self::get_input_amount(
						&bought_token_amount,
						&bought_asset_exchange.token_a_reserve,
						&bought_asset_exchange.token_b_reserve,
						&bought_asset_exchange.fee_numerator,
						&bought_asset_exchange.fee_denominator,
					)?;
					let sold_token_amount = Self::get_input_amount(
						&currency_amount,
						&sold_asset_exchange.token_a_reserve,
						&sold_asset_exchange.token_b_reserve,
						&sold_asset_exchange.fee_numerator,
						&sold_asset_exchange.fee_denominator,
					)?;
					ensure!(sold_token_amount <= max_sold_tokens, Error::<T>::MaxSoldTokensTooLow);

					Ok((
						sold_token_amount,   // should be a Balance
						bought_token_amount, // should be a Balance
					))
				},
			}
		}

		fn check_trade_amount<A: Zero, B: Zero>(
			amount: &TradeAmount<A, B>,
		) -> Result<(), Error<T>> {
			match amount {
				TradeAmount::FixedInput { input_amount, min_output } => {
					ensure!(!input_amount.is_zero(), Error::TradeAmountIsZero);
					ensure!(!min_output.is_zero(), Error::TradeAmountIsZero);
				},
				TradeAmount::FixedOutput { output_amount, max_input } => {
					ensure!(!output_amount.is_zero(), Error::TradeAmountIsZero);
					ensure!(!max_input.is_zero(), Error::TradeAmountIsZero);
				},
			};
			Ok(())
		}

		fn swap_asset_for_asset(
			sold_asset_exchange: ExchangeOf<T>,
			bought_asset_exchange: ExchangeOf<T>,
			sold_token_amount: BalanceOf<T>,
			bought_token_amount: BalanceOf<T>,
			buyer: AccountIdOf<T>,
			recipient: AccountIdOf<T>,
		) -> DispatchResult {
			let pallet_account: AccountIdOf<T> = T::pallet_account();
			Ok(())
		}

		   // Update the reserves of an exchange after a sell operation
		   fn update_reserve_after_sell(
			exchange: &Exchange<T>,
			sold_amount: BalanceOf<T>,
			is_token_a: bool
		) -> Result<Exchange<T>, DispatchError> {
			let mut updated_exchange = exchange.clone();
	
			if is_token_a {
				updated_exchange.token_a_reserve = Self::reduce_reserve(exchange.token_a_reserve, sold_amount)?;
			} else {
				updated_exchange.token_b_reserve = Self::reduce_reserve(exchange.token_b_reserve, sold_amount)?;
			}
			
			Ok(updated_exchange)
		}
	
		// Update the reserves of an exchange after a buy operation
		fn update_reserve_after_buy(
			exchange: &Exchange<T>,
			bought_amount: BalanceOf<T>,
			is_token_a: bool
		) -> Result<Exchange<T>, DispatchError> {
			let mut updated_exchange = exchange.clone();
	
			if is_token_a {
				updated_exchange.token_a_reserve = Self::increase_reserve(exchange.token_a_reserve, bought_amount)?;
			} else {
				updated_exchange.token_b_reserve = Self::increase_reserve(exchange.token_b_reserve, bought_amount)?;
			}
	
			Ok(updated_exchange)
		}
	
		// Update the exchange in storage
		fn update_exchange_storage(
			token_pair: (T::AccountId, T::AccountId),
			exchange: Exchange<T>
		) {
			Exchanges::<T>::insert(&token_pair, exchange);
		}
	
		// Decrease reserve, checking for underflow
		fn reduce_reserve(
			reserve: BalanceOf<T>, 
			amount: BalanceOf<T>
		) -> Result<BalanceOf<T>,  Error<T>> {
			reserve.checked_sub(&amount).ok_or(Error::<T>::ArithmeticUnderflow)
		}
	
		// Increase reserve, checking for overflow
		fn increase_reserve(
			reserve: BalanceOf<T>, 
			amount: BalanceOf<T>
		) -> Result<BalanceOf<T>,  Error<T>> {
			reserve.checked_add(&amount).ok_or(Error::<T>::ArithmeticOverflow)
		}
	}
}
