//! All the traits exposed to be used in other custom modules

/// Tree trait definition to be used in other pallets
pub trait TreeInterface<AccountId, TreeId, Element> {
	// Creates a new tree
	fn create(creator: AccountId, depth: u8) -> Result<TreeId, Error>;
	/// Adds members/leaves to the tree
	fn insert_in_order(id: TreeId, leaf: Element) -> Result<Element, Error>;
}

/// Tree trait for inspecting tree state
pub trait TreeInspector<AccountId, TreeId, Element> {
	/// Gets the merkle root for a tree or returns `TreeDoesntExist`
	fn get_root(id: TreeId) -> Result<Element, Error>;
	/// Checks if a merkle root is in a tree's cached history or returns
	/// `TreeDoesntExist
	fn is_known_root(id: TreeId, target: Element) -> Result<bool, Error>;
}
