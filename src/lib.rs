#![feature(ptr_internals, maybe_uninit_slice)]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate alloc;

mod front_string;
mod front_vec;
pub mod iter;

#[cfg(test)]
mod tests;

pub use crate::{front_string::FrontString, front_vec::FrontVec};
