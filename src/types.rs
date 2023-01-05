use std::ffi::CString;
use std::net::IpAddr;

use crate::binding;

/// All the supported ipset types.
/// TODO hash:net is not fully supported now.
pub enum SetType {
    HashIp,
    HashNet,
}

impl SetType {
    /// get string name
    pub(crate) fn to_cstring(&self) -> CString {
        match self {
            SetType::HashIp => CString::new("hash:ip").unwrap(),
            SetType::HashNet => CString::new("hash:net").unwrap(),
        }
    }
}

/// Ip wrapper including ipv4 and ipv6
pub enum CIpAddr {
    IPv4(libc::in_addr),
    IPv6(libc::in6_addr),
}

impl CIpAddr {
    /// get ip address pointer and ip family pointer.
    pub fn as_ptr(&self) -> (*const std::ffi::c_void, *const std::ffi::c_void) {
        match self {
            CIpAddr::IPv4(ip) => (ip as *const _ as _, &binding::NFPROTO_IPV4 as *const _ as _),
            CIpAddr::IPv6(ip) => (ip as *const _ as _, &binding::NFPROTO_IPV6 as *const _ as _),
        }
    }
}

/// create a `CIpAddr` from `IpAddr`
pub(crate) fn get_caddr(ip: IpAddr) -> CIpAddr {
    match ip {
        IpAddr::V4(v4) => {
            let ip: u32 = v4.into();
            CIpAddr::IPv4(libc::in_addr { s_addr: ip.to_be() })
        }
        IpAddr::V6(v6) => CIpAddr::IPv6(libc::in6_addr {
            s6_addr: v6.octets(),
        }),
    }
}

/// Errors defined in this crate.
#[derive(Debug)]
pub enum Error {
    DataSet(String, bool),
    Cmd(String, bool),
    TypeGet(String, bool),
}

impl Error {
    pub(crate) fn cmd_contains(&self, m: &str) -> bool {
        if let Error::Cmd(message, _) = self {
            message.contains(m)
        } else {
            false
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            Error::DataSet(_, error) => *error,
            Error::Cmd(_, error) => *error,
            Error::TypeGet(_, error) => *error,
        }
    }
}
