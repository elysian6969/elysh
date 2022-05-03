#![feature(char_indices_offset)]
//#![no_std]

pub(crate) use common::Common;

pub use args::{Arg, Args};
pub use chars::Chars;
pub use command::{Command, CommandError};
pub use quote::Quote;
pub use token::Token;
pub use value::Value;
pub use vars::{Var, Vars};

mod args;
mod chars;
mod command;
mod quote;
mod token;
mod value;
mod vars;

pub(crate) mod common;
