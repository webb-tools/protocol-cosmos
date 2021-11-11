// TODO find out why we need this attribute? in order to compile for wasm?
#![cfg_attr(not(feature = "std"), no_std)]


pub mod anchor;
pub mod merkle_tree;
pub mod linkable_tree;