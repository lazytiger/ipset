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
//!use ipset::types::{Error, HashIp};
//!use ipset::Session;
//!use ipset::IPSet;
//!
//!fn test() -> Result<(), Error> {
//!    let mut session: Session<HashIp> = Session::<HashIp>::new("test".to_string());
//!    let ip: IpAddr = "192.168.3.1".parse().unwrap();
//!    session.create(|builder| builder.with_ipv6(false)?.build())?;
//!
//!    let ret = session.add(ip, None)?;
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
//!    let ret = session.save("test.ipset".into()).unwrap();
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
#![feature(c_variadic)]
#![feature(concat_idents)]

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

unsafe fn _ipset_store(filename: String) -> std::ffi::c_int {
    let filename = std::ffi::CString::new(filename).unwrap();
    binding::ipset_load_types();
    let set = binding::ipset_init();
    let session = binding::ipset_session(set);
    let ret = binding::ipset_session_io_normal(
        session,
        filename.as_ptr(),
        binding::ipset_io_type_IPSET_IO_INPUT,
    );
    if ret < 0 {
        return ret;
    }

    let file = binding::ipset_session_io_stream(session, binding::ipset_io_type_IPSET_IO_INPUT);
    return binding::ipset_parse_stream(set, file);
}
