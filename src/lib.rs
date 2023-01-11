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
//! * BitmapIp
//! * BitmapIpMac
//! * BitmapPort
//! * HashIp
//! * HashIpMac
//! * HashIpMark
//! * HashIpPort
//! * HashIpPortIp
//! * HashIpPortNet
//! * HashMac
//! * HashNet
//! * HashNetIface
//! * HashNetNet
//! * HashNetPort
//! * HashNetPortNet
//! * ListSet
//!
//! # Example
//! ```rust
//!use std::net::IpAddr;
//!
//!use ipset::{Error, HashIp, Session};
//!
//!fn test() -> Result<(), Error> {
//!    let mut session: Session<HashIp> = Session::<HashIp>::new("test".to_string());
//!    let ip: IpAddr = "192.168.3.1".parse().unwrap();
//!    session.create(|builder| builder.with_ipv6(false)?.build())?;
//!
//!    let ret = session.add(ip)?;
//!    println!("add {}", ret);
//!
//!    let exists = session.test(ip)?;
//!    println!("test {}", exists);
//!
//!    let ips = session.list()?;
//!    for ip in ips {
//!        println!("list {}", ip);
//!    }
//!
//!    let ret = session.del(ip)?;
//!    println!("del {}", ret);
//!
//!    let ret = session.flush()?;
//!    println!("flush {}", ret);
//!
//!    let ret = session.destroy()?;
//!    println!("destroy {}", ret);
//!
//!    Ok(())
//!}
//!
//! fn main() {
//!     if let Err(err) = test() {
//!         println!("{:?}", err);
//!     }
//! }
//! ```
#![feature(c_variadic)]
#![feature(concat_idents)]

pub use session::{CreateBuilder, Session};
pub use types::Error;

pub use crate::types::{
    BitmapIp, BitmapIpMac, BitmapPort, HashIp, HashIpMac, HashIpMark, HashIpPort, HashIpPortIp,
    HashIpPortNet, HashMac, HashNet, HashNetIface, HashNetNet, HashNetPort, HashNetPortNet,
    IfaceDataType, IpDataType, ListSet, MacDataType, MarkDataType, NetDataType, Parse,
    PortDataType, SetData, SetDataType, SetType,
};

#[allow(non_camel_case_types)]
#[allow(unused)]
#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
mod binding;
mod session;
mod types;
