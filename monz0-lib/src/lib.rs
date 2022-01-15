//! A library for scripting cash transfers between Monzo pots

#![feature(type_alias_impl_trait)]
#![deny(
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations,
    missing_docs
)]
#![warn(clippy::pedantic)]

pub use monzo::Pot;
mod ledger;
pub use ledger::Ledger;
mod client;
pub mod state;
#[doc(inline)]
pub use state::State;
pub mod operation;
pub use client::{Auth, Client};
#[doc(inline)]
pub use operation::Operation;
