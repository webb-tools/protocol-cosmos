
use sp_std::vec::Vec;
use std::error::Error;

 // TODO implement error handing e.g. use library thiserror

pub trait AnchorConfig {
	type LeafIndex;
	type AccountId;
	type Balance;
	// removed CurrencyId, it does not make sense in cosmos
	type ChainId;
	type TreeId;
	type Element; // implement codeccin cosmos
}

/// Anchor trait definition to be used in other pallets
pub trait AnchorInterface<C: AnchorConfig> {
	// Creates a new anchor
	fn create(
		creator: C::AccountId,
		deposit_size: C::Balance,
		depth: u8,
		max_edges: u32,
        // TODO replace asset in cosmos we have Coin and Denom
		asset: C::CurrencyId,
	) -> Result<C::TreeId, Error>;
	/// Deposit into the anchor
	fn deposit(account: C::AccountId, id: C::TreeId, leaf: C::Element) -> Result<(), Error>;
	/// Withdraw from the anchor
	fn withdraw(
		id: C::TreeId,
		proof_bytes: &[u8],
		chain_id: C::ChainId,
		roots: Vec<C::Element>,
		nullifier_hash: C::Element,
		recipient: C::AccountId,
		relayer: C::AccountId,
		fee: C::Balance,
		refund: C::Balance,
	) -> Result<(), std::Error>;
	// Stores nullifier hash from a spend tx
	fn add_nullifier_hash(id: C::TreeId, nullifier_hash: C::Element) -> Result<(), Error>;
	/// Add an edge to this tree
	fn add_edge(
		id: C::TreeId,
		src_chain_id: C::ChainId,
		root: C::Element,
		latest_leaf_index: C::LeafIndex,
	) -> Result<(), Error>;
	/// Update an edge for this tree
	fn update_edge(
		id: C::TreeId,
		src_chain_id: C::ChainId,
		root: C::Element,
		latest_leaf_index: C::LeafIndex,
	) -> Result<(), Error>;
}

// TODO add import pallet_linkable_tree::Config
pub trait Config<I: 'static = ()>: frame_system::Config + pallet_linkable_tree::Config<I> {
	/// The overarching event type.
    // TODO I guess i have to use cosmos Event here
	type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

    // I am not sure if it does make sense
	//#[pallet::constant]
	//type PalletId: Get<PalletId>;

	/// The tree type
	type LinkableTree: LinkableTreeInterface<pallet_linkable_tree::LinkableTreeConfigration<Self, I>>
		+ LinkableTreeInspector<pallet_linkable_tree::LinkableTreeConfigration<Self, I>>;
   
	/// The verifier
	//type Verifier: VerifierModule;

	/// Currency type for taking deposits
	//type Currency: MultiCurrency<Self::AccountId>;

    // TODO research what it is used for 
	//type PostDepositHook: PostDepositHook<Self, I>;

    // TODO some other id from cosmos instead 
	/// Native currency id
	//#[pallet::constant]
	//type NativeCurrencyId: Get<CurrencyIdOf<Self, I>>;

	/// Weight info for pallet
	type WeightInfo: WeightInfo;
}

impl<T: Config<I>, I: 'static> AnchorInterface<AnchorConfigration<T, I>> for Pallet<T, I> {
	fn create(
		creator: T::AccountId,
		deposit_size: BalanceOf<T, I>,
		depth: u8,
		max_edges: u32,
        // TODO replace asset with cosmos type
		asset: CurrencyIdOf<T, I>,
	) -> Result<T::TreeId, Error> {
		let id = T::LinkableTree::create(creator.clone(), max_edges, depth)?;
		// TODO implement Anchors for cosmos
        Anchors::<T, I>::insert(
			id,
			Some(AnchorMetadata {
				creator,
				deposit_size,
				asset,
			}),
		);
		Ok(id)
	}

	fn deposit(depositor: T::AccountId, id: T::TreeId, leaf: T::Element) -> Result<(), Error> {
		// insert the leaf
		T::LinkableTree::insert_in_order(id, leaf)?;

		let anchor = Self::get_anchor(id)?;
		// transfer tokens to the pallet
        // TODO reimplement transfer() for cosmos
		<T as Config<I>>::Currency::transfer(anchor.asset, &depositor, &Self::account_id(), anchor.deposit_size)?;

		Ok(())
	}

    // TODO reimpllement it for cosmos
	fn withdraw(
		id: T::TreeId,
		proof_bytes: &[u8],
		chain_id: T::ChainId,
		roots: Vec<T::Element>,
		nullifier_hash: T::Element,
		recipient: T::AccountId,
		relayer: T::AccountId,
		fee: BalanceOf<T, I>,
		refund: BalanceOf<T, I>,
	) -> Result<(), Error> {
		// double check the number of roots
		T::LinkableTree::ensure_max_edges(id, roots.len())?;
		// Check if local root is known
		T::LinkableTree::ensure_known_root(id, roots[0])?;
		// Check if neighbor roots are known
		T::LinkableTree::ensure_known_neighbor_roots(id, &roots)?;

		// Check nullifier and add or return `InvalidNullifier`
		Self::ensure_nullifier_unused(id, nullifier_hash)?;
		Self::add_nullifier_hash(id, nullifier_hash)?;
		// Format proof public inputs for verification
		// FIXME: This is for a specfic gadget so we ought to create a generic handler
		// FIXME: Such as a unpack/pack public inputs trait
		// FIXME: 	-> T::PublicInputTrait::validate(public_bytes: &[u8])
		//
		// nullifier_hash (0..32)
		// recipient (32..64)
		// relayer (64..96)
		// fee (96..128)
		// refund (128..160)
		// chain_id (160..192)
		// root[1] (224..256)
		// root[2] (256..288)
		// root[m - 1] (...)
		let mut bytes = vec![];

		let element_encoder = |v: &[u8]| {
			let mut output = [0u8; 32];
			output.iter_mut().zip(v).for_each(|(b1, b2)| *b1 = *b2);
			output
		};
		let recipient_bytes = truncate_and_pad(&recipient.using_encoded(element_encoder)[..]);
		let relayer_bytes = truncate_and_pad(&relayer.using_encoded(element_encoder)[..]);
		let fee_bytes = fee.using_encoded(element_encoder);
		let refund_bytes = refund.using_encoded(element_encoder);
		let chain_id_bytes = chain_id.using_encoded(element_encoder);

		bytes.extend_from_slice(&nullifier_hash.encode());
		bytes.extend_from_slice(&recipient_bytes);
		bytes.extend_from_slice(&relayer_bytes);
		bytes.extend_from_slice(&fee_bytes);
		bytes.extend_from_slice(&refund_bytes);
		bytes.extend_from_slice(&chain_id_bytes);
		for i in 0..roots.len() {
			bytes.extend_from_slice(&roots[i].encode());
		}
		let result = <T as pallet::Config<I>>::Verifier::verify(&bytes, proof_bytes)?;
		ensure!(result, Error::<T, I>::InvalidWithdrawProof);
		// transfer the assets
		let anchor = Self::get_anchor(id)?;
		<T as Config<I>>::Currency::transfer(anchor.asset, &Self::account_id(), &recipient, anchor.deposit_size)?;
		Ok(())
	}

	fn add_nullifier_hash(id: T::TreeId, nullifier_hash: T::Element) -> Result<(), Error> {
		NullifierHashes::<T, I>::insert(id, nullifier_hash, true);
		Ok(())
	}

	fn add_edge(
		id: T::TreeId,
		src_chain_id: T::ChainId,
		root: T::Element,
		latest_leaf_index: T::LeafIndex,
	) -> Result<(), Error> {
		T::LinkableTree::add_edge(id, src_chain_id, root, latest_leaf_index)
	}

	fn update_edge(
		id: T::TreeId,
		src_chain_id: T::ChainId,
		root: T::Element,
		latest_leaf_index: T::LeafIndex,
	) -> Result<(), Error> {
		T::LinkableTree::update_edge(id, src_chain_id, root, latest_leaf_index)
	}
}

/// Anchor trait for inspecting tree state
pub trait AnchorInspector<C: AnchorConfig> {
	/// Check if a nullifier has been used in a tree or returns
	/// `InvalidNullifier`
	fn is_nullifier_used(id: C::TreeId, nullifier: C::Element) -> bool;
	/// Check if a nullifier has been used in a tree and throws if not
	fn ensure_nullifier_unused(id: C::TreeId, nullifier: C::Element) -> Result<(), Error>;
	/// Check if this linked tree has this edge (for backwards compatability)
	fn has_edge(id: C::TreeId, src_chain_id: C::ChainId) -> bool;
}
