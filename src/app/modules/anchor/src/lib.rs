use crate::bank::{AccountId, Balances};

use darkwebb_primitives::{
    anchor::{AnchorConfig, AnchorInspector, AnchorInterface},
    linkable_tree::{LinkableTreeInspector, LinkableTreeInterface},
};

// TODO I have not found this trait Store in substrate protocol codebase, what is doing?
#[pallet::generate_store(pub(super) trait Store)]
pub struct Pallet<T, I = ()>(_);

pub trait Config<I: 'static = ()>: frame_system::Config + linkable_tree::Config<I> {
    // TODO I guess i have to use cosmos Event here
    //type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

    //#[pallet::constant]
    //type PalletId: Get<PalletId>;

    // The tree type
    type LinkableTree: LinkableTreeInterface<linkable_tree::LinkableTreeConfigration<Self, I>>
        + LinkableTreeInspector<linkable_tree::LinkableTreeConfigration<Self, I>>;

    // The verifier
    // I am not sure I need it for now
    //type Verifier: VerifierModule;

    // TODO there is such a type in cosmos i think
    // type Currency: MultiCurrency<Self::AccountId>;

    //type PostDepositHook: PostDepositHook<Self, I>;

    //Native currency id does not relate to cosmos
    //#[pallet::constant]
    //type NativeCurrencyId: Get<CurrencyIdOf<Self, I>>;

    // Weight info id does not relate to cosmos
    // Weight info for pallet
    //type WeightInfo: WeightInfo;
}

// TODO use cosmos KVStore (key value storage) instead of substrate StorageMap
// TODO more research on substrate storage
/// The map of trees to their anchor metadata
#[pallet::storage]
#[pallet::getter(fn anchors)]
pub type Anchors<T: Config<I>, I: 'static = ()> = StorageMap<
    _,
    Blake2_128Concat,
    T::TreeId,
    Option<AnchorMetadata<T::AccountId, BalanceOf<T, I>, CurrencyIdOf<T, I>>>,
    ValueQuery,
>;

/// The map of trees to their spent nullifier hashes
#[pallet::storage]
#[pallet::getter(fn nullifier_hashes)]
pub type NullifierHashes<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::TreeId,
    Blake2_128Concat,
    T::Element,
    bool,
    ValueQuery,
>;

// TODO where can I find the implementation of these attributes?
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config<I>, I: 'static = ()> {
    /// New tree created
    AnchorCreation { tree_id: T::TreeId },
}

#[pallet::error]
pub enum Error<T, I = ()> {
    /// Invalid Merkle Roots
    InvalidMerkleRoots,
    /// Unknown root
    UnknownRoot,
    /// Invalid withdraw proof
    InvalidWithdrawProof,
    /// Mixer not found.
    NoAnchorFound,
    /// Invalid nullifier that is already used
    /// (this error is returned when a nullifier is used twice)
    AlreadyRevealedNullifier,
}

// TODO investigate why we need it
#[pallet::hooks]
impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {}

// TODO investigate why we need it
/*
#[pallet::call]
impl<T: Config<I>, I: 'static> Pallet<T, I> {
    #[pallet::weight(<T as Config<I>>::WeightInfo::create(*depth as u32, *max_edges))]
    pub fn create(
        origin: OriginFor<T>,
        deposit_size: BalanceOf<T, I>,
        max_edges: u32,
        depth: u8,
        asset: CurrencyIdOf<T, I>,
    ) -> DispatchResultWithPostInfo {
        // Should it only be the root who can create anchors?
        ensure_root(origin)?;
        let tree_id =
            <Self as AnchorInterface<_>>::create(T::AccountId::default(), deposit_size, depth, max_edges, asset)?;
        Self::deposit_event(Event::AnchorCreation { tree_id });
        Ok(().into())
    }

    #[pallet::weight(<T as Config<I>>::WeightInfo::deposit())]
    pub fn deposit(origin: OriginFor<T>, tree_id: T::TreeId, leaf: T::Element) -> DispatchResultWithPostInfo {
        let origin = ensure_signed(origin)?;
        <Self as AnchorInterface<_>>::deposit(origin.clone(), tree_id, leaf)?;
        T::PostDepositHook::post_deposit(origin, tree_id, leaf)?;
        Ok(().into())
    }

    #[pallet::weight(<T as Config<I>>::WeightInfo::withdraw())]
    pub fn withdraw(
        origin: OriginFor<T>,
        id: T::TreeId,
        proof_bytes: Vec<u8>,
        chain_id: T::ChainId,
        roots: Vec<T::Element>,
        nullifier_hash: T::Element,
        recipient: T::AccountId,
        relayer: T::AccountId,
        fee: BalanceOf<T, I>,
        refund: BalanceOf<T, I>,
    ) -> DispatchResultWithPostInfo {
        ensure_signed(origin)?;
        <Self as AnchorInterface<_>>::withdraw(
            id,
            proof_bytes.as_slice(),
            chain_id,
            roots,
            nullifier_hash,
            recipient,
            relayer,
            fee,
            refund,
        )?;
        Ok(().into())
    }
}
}
*/

pub struct AnchorConfigration<T: Config<I>, I: 'static>(
    core::marker::PhantomData<T>,
    core::marker::PhantomData<I>,
);

impl<T: Config<I>, I: 'static> AnchorConfig for AnchorConfigration<T, I> {
    type AccountId = T::AccountId;
    type Balance = T::Balance;
    type ChainId = T::ChainId;
    type Element = T::Element;
    type LeafIndex = T::LeafIndex;
    type TreeId = T::TreeId;
}

impl<T: Config<I>, I: 'static> AnchorInterface<AnchorConfigration<T, I>> for Pallet<T, I> {
    fn create(
        creator: T::AccountId,
        deposit_size: T::Balance,
        depth: u8,
        max_edges: u32,
        // TODO we have to come up with some identifier instead of asset
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
        <T as Config<I>>::Currency::transfer(
            anchor.asset,
            &depositor,
            &Self::account_id(),
            anchor.deposit_size,
        )?;

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
        // FIXME:   -> T::PublicInputTrait::validate(public_bytes: &[u8])
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
        <T as Config<I>>::Currency::transfer(
            anchor.asset,
            &Self::account_id(),
            &recipient,
            anchor.deposit_size,
        )?;
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

impl<T: Config<I>, I: 'static> AnchorInspector<AnchorConfigration<T, I>> for Pallet<T, I> {
    fn is_nullifier_used(tree_id: T::TreeId, nullifier_hash: T::Element) -> bool {
        // TODO reimplement using cosmos storage
        NullifierHashes::<T, I>::contains_key(tree_id, nullifier_hash)
    }

    fn ensure_nullifier_unused(id: T::TreeId, nullifier: T::Element) -> Result<(), DispatchError> {
        ensure!(
            !Self::is_nullifier_used(id, nullifier),
            Error::<T, I>::AlreadyRevealedNullifier
        );
        Ok(())
    }

    fn has_edge(id: T::TreeId, src_chain_id: T::ChainId) -> bool {
        T::LinkableTree::has_edge(id, src_chain_id)
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    pub fn get_anchor(
        id: T::TreeId,
    ) -> Result<AnchorMetadata<T::AccountId, BalanceOf<T, I>, CurrencyIdOf<T, I>>, DispatchError>
    {
        let anchor = Anchors::<T, I>::get(id);
        ensure!(anchor.is_some(), Error::<T, I>::NoAnchorFound);
        Ok(anchor.unwrap())
    }
}

// TODO investigate if we have to used it in cosmos
pub trait PostDepositHook<T: Config<I>, I: 'static> {
    fn post_deposit(depositor: T::AccountId, id: T::TreeId, leaf: T::Element) -> DispatchResult;
}

// TODO investigate if we have to used it in cosmos
impl<T: Config<I>, I: 'static> PostDepositHook<T, I> for () {
    fn post_deposit(_: T::AccountId, _: T::TreeId, _: T::Element) -> DispatchResult {
        Ok(())
    }
}

// TODO what is that?
/// Truncate and pad 256 bit slice
pub fn truncate_and_pad(t: &[u8]) -> Vec<u8> {
    let mut truncated_bytes = t[..20].to_vec();
    truncated_bytes.extend_from_slice(&[0u8; 12]);
    truncated_bytes
}
