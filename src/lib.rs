#![feature(c_variadic)]

use std::ffi::{CStr, CString};
use std::net::IpAddr;

use crate::binding::{ipset_cmd, ipset_opt};

#[allow(non_camel_case_types)]
#[allow(unused)]
#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
mod binding;

pub struct IPSet {
    set: *mut binding::ipset,
}

pub struct Session {
    session: *mut binding::ipset_session,
    data: *mut binding::ipset_data,
}

#[no_mangle]
pub unsafe extern "C" fn custom_error(
    ipset: *mut binding::ipset,
    p: *mut ::std::os::raw::c_void,
    status: ::std::os::raw::c_int,
    msg: *const ::std::os::raw::c_char,
    mut args: ...
) -> std::ffi::c_int {
    let content = args.arg::<*const std::ffi::c_char>();
    let content = CStr::from_ptr(content);
    println!("{:?}", content);
    let mut buffer = vec![0u8; 1024];
    let n = libc::sprintf(buffer.as_mut_ptr() as _, msg, args);
    let msg = CStr::from_ptr(msg);
    println!("{:?}", msg);
    buffer[n as usize] = 0;
    buffer.resize(n as usize, 0);
    println!("{}, error:{}", n, String::from_utf8(buffer).unwrap());
    n
}

#[no_mangle]
pub unsafe extern "C" fn outfn(
    session: *mut binding::ipset_session,
    p: *mut ::std::os::raw::c_void,
    fmt: *const ::std::os::raw::c_char,
    args: ...
) -> ::std::os::raw::c_int {
    let mut buffer = vec![0u8; 1024];
    let n = libc::sprintf(buffer.as_mut_ptr() as _, fmt, args);
    println!("output:{}", String::from_utf8(buffer).unwrap());
    0
}

impl IPSet {
    pub fn new() -> IPSet {
        unsafe {
            binding::ipset_load_types();
            let set = binding::ipset_init();
            binding::ipset_custom_printf(
                set,
                Some(custom_error),
                None,
                Some(outfn),
                std::ptr::null_mut(),
            );
            IPSet { set }
        }
    }

    pub fn session(&self) -> Session {
        unsafe {
            let session = binding::ipset_session(self.set);
            let data = binding::ipset_session_data(session);
            Session { session, data }
        }
    }
}

pub enum CIpAddr {
    IPv4(libc::in_addr),
    IPv6(libc::in6_addr),
}

impl CIpAddr {
    pub fn as_ptr(&self) -> (*const std::ffi::c_void, *const std::ffi::c_void) {
        match self {
            CIpAddr::IPv4(ip) => (ip as *const _ as _, &binding::NFPROTO_IPV4 as *const _ as _),
            CIpAddr::IPv6(ip) => (ip as *const _ as _, &binding::NFPROTO_IPV6 as *const _ as _),
        }
    }
}

fn get_caddr(ip: IpAddr) -> CIpAddr {
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

#[derive(Debug)]
pub enum Error {
    DataSet(String),
    Cmd(String),
    TypeGet(String),
}

impl Session {
    fn set_data(
        &self,
        opt: binding::ipset_opt,
        value: *const std::ffi::c_void,
    ) -> Result<(), Error> {
        unsafe {
            if binding::ipset_data_set(self.data, opt, value) < 0 {
                let err = binding::ipset_session_report_msg(self.session);
                let err = CStr::from_ptr(err).to_string_lossy().to_string();
                Err(Error::DataSet(err))
            } else {
                Ok(())
            }
        }
    }

    fn error(&self) -> String {
        unsafe {
            let err = binding::ipset_session_report_msg(self.session);
            let err = CStr::from_ptr(err).to_string_lossy().to_string();
            err
        }
    }

    fn run_cmd(&self, cmd: binding::ipset_cmd) -> Result<(), Error> {
        unsafe {
            if binding::ipset_cmd(self.session, cmd, 0) < 0 {
                Err(Error::Cmd(self.error()))
            } else {
                Ok(())
            }
        }
    }

    fn ip_cmd(&self, name: &str, ip: IpAddr, cmd: binding::ipset_cmd) -> Result<(), Error> {
        unsafe {
            let name = CString::new(name).unwrap();
            self.set_data(binding::ipset_opt_IPSET_SETNAME, name.as_ptr() as _)?;

            let addr = get_caddr(ip);
            let (ip, family) = addr.as_ptr();
            self.set_data(binding::ipset_opt_IPSET_OPT_FAMILY, family)?;

            let typ = binding::ipset_type_get(self.session, binding::ipset_cmd_IPSET_CMD_TEST);
            if typ.is_null() {
                return Err(Error::TypeGet(self.error()));
            }

            self.set_data(binding::ipset_opt_IPSET_OPT_IP, ip)?;

            self.run_cmd(cmd)
        }
    }

    pub fn test(&self, name: &str, ip: IpAddr) -> Result<bool, Error> {
        match self.ip_cmd(name, ip, binding::ipset_cmd_IPSET_CMD_TEST) {
            Ok(_) => Ok(true),
            Err(err) => unsafe {
                if binding::ipset_session_report_type(self.session)
                    != binding::ipset_err_type_IPSET_ERROR
                {
                    Ok(false)
                } else {
                    Err(err)
                }
            },
        }
    }

    pub fn add(&self, name: &str, ip: IpAddr) -> Result<bool, Error> {
        self.ip_cmd(name, ip, binding::ipset_cmd_IPSET_CMD_ADD)
            .map(|_| true)
    }
}

impl Drop for IPSet {
    fn drop(&mut self) {
        unsafe {
            if !self.set.is_null() {
                binding::ipset_fini(self.set);
            }
        }
    }
}
