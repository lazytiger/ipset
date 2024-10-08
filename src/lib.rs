//! A library wrapper for `libipset`.  
//! Support the following commands:
//! * add
//! * del
//! * test
//! * create
//! * list
//! * destroy
//! * flush
//! * save
//! * restore
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
//! ```rust,no_run
//!use std::net::IpAddr;
//!
//!use ipset::types::{AddOption, BitmapIp, EnvOption, Error, HashIp, IpDataType, ListResult};
//!use ipset::{IPSet, Session};
//!
//!fn test() -> Result<(), Error> {
//!    let mut session: Session<HashIp> = Session::<HashIp>::new("test".to_string());
//!    let ip: IpAddr = "192.168.3.1".parse()?;
//!    session.create(|builder| builder.with_ipv6(false)?.build())?;
//!
//!    let ret = session.add(ip, &[])?;
//!    println!("add {}", ret);
//!
//!    let exists = session.test(ip)?;
//!    println!("test {}", exists);
//!
//!    let ips = session.list()?;
//!    match ips {
//!         ListResult::Normal(ret) => {
//!             println!("name:{}, type:{}, revision:{}, size_in_memory:{}, references:{}, entry_size:{}, header:{:?}",
//!                 ret.name, ret.typ, ret.revision, ret.size_in_memory, ret.references, ret.entry_size, ret.header)
//!         }
//!         ListResult::Terse(names) => {
//!             println!("{:?}", names);
//!         }
//!   }
//!    session.set_option(EnvOption::ListSetName);
//!    let ips = session.list()?;
//!    session.unset_option(EnvOption::ListSetName);
//!    match ips {
//!         ListResult::Normal(ret) => {
//!             println!("name:{}, type:{}, revision:{}, size_in_memory:{}, references:{}, entry_size:{}, header:{:?}",
//!                 ret.name, ret.typ, ret.revision, ret.size_in_memory, ret.references, ret.entry_size, ret.header)
//!         }
//!         ListResult::Terse(names) => {
//!             println!("{:?}", names);
//!         }
//!    }
//!
//!    let ret = session.save("test.ipset".into())?;
//!    println!("save {}", ret);
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
//!
//!     if let Err(err) = test() {
//!         println!("{:?}", err);
//!     }
//!
//!     let set = IPSet::new();
//!     set.restore("test.ipset".to_string()).unwrap();
//! }
//! ```

pub use session::{CreateBuilder, Session};
pub use set::IPSet;

#[allow(non_camel_case_types)]
#[allow(unused)]
#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
mod binding;
mod session;
mod set;
pub mod types;
