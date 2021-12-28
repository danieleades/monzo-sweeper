//! A library for scripting cash transfers between Monzo pots

#![feature(type_alias_impl_trait)]
#![deny(
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations,
    missing_docs
)]
#![warn(clippy::pedantic)]

mod client;
pub mod state;
#[doc(inline)]
pub use state::State;
pub mod operation;
#[doc(inline)]
pub use operation::Operation;
mod transactions;

pub use client::{Auth, Client};
pub use monzo::Pot;
pub use transactions::{Ledger, Transactions};