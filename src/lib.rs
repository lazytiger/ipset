//! A library wrapper for `libipset`.  
//! Support the following commands:
//! * add
//! * del
//! * test
//! * create
//! * list
//! * destroy
//! * flush
//!
//! Support the following type:
//! * hash:ip
//! * hash:net not fully supported
//!
//! # Example
//! ```rust
//! use ipset::{Session, SetType};
//!
//! fn main() {
//!     let mut session = Session::new();
//!     if let Err(err) = session.create("test", SetType::HashIp, |builder| {
//!         builder.with_ipv6(false)?.build()
//!     }) {
//!         println!("create ipset failed:{:?}", err);
//!         return;
//!     }
//!
//!     if let Err(err) = session.add("test", "127.0.0.1".parse().unwrap()) {
//!         println!("add ip to ipset failed:{:?}", err);
//!         return;
//!    }
//!
//!     if let Err(err) = session.list("test") {
//!         println!("list ip from ipset failed:{:?}", err);
//!         return;
//!     }
//!
//!     if let Err(err) = session.del("test", "127.0.0.1".parse().unwrap()) {
//!         println!("delete ip from ipset failed:{:?}", err);
//!         return;
//!     }
//!
//!     if let Err(err) = session.flush("test") {
//!         println!("flush ipset failed:{:?}", err);
//!     }
//!
//!     if let Err(err) = session.destroy("test") {
//!         println!("destroy ipset failed:{:?}", err);
//!     }
//! }
//! ```
#![feature(c_variadic)]

pub use session::{CreateBuilder, Session};
pub use types::{Error, SetType};

use crate::types::{HashIp, HashNet};

pub type SessionHashIp = Session<HashIp>;
pub type SessionHashNet = Session<HashNet>;

#[allow(non_camel_case_types)]
#[allow(unused)]
#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
mod binding;
mod session;
mod types;
