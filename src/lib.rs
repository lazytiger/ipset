#![feature(c_variadic)]

use std::ffi::{CStr, CString};
use std::net::IpAddr;

#[allow(non_camel_case_types)]
#[allow(unused)]
#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
mod binding;

pub struct IPSet {
    set: *mut binding::ipset,
    output: Vec<String>,
}

pub struct Session<'a> {
    session: *mut binding::ipset_session,
    data: *mut binding::ipset_data,
    set: &'a mut IPSet,
}

#[no_mangle]
pub unsafe extern "C" fn outfn(
    _session: *mut binding::ipset_session,
    p: *mut ::std::os::raw::c_void,
    fmt: *const ::std::os::raw::c_char,
    mut args: ...
) -> ::std::os::raw::c_int {
    let raw = args.arg::<*const std::ffi::c_char>();
    let data = CStr::from_ptr(raw);
    let len = data.to_bytes().len();
    if len == 0 {
        return 0;
    }
    let buffer = vec![0u8; len];
    libc::sprintf(buffer.as_ptr() as _, fmt, raw);
    let set = (p as *mut IPSet).as_mut().unwrap();
    set.output.push(String::from_utf8_unchecked(buffer));
    0
}

impl IPSet {
    pub fn new() -> IPSet {
        unsafe {
            binding::ipset_load_types();
            let set = binding::ipset_init();
            IPSet {
                set,
                output: Default::default(),
            }
        }
    }

    pub fn session(&mut self) -> Session {
        unsafe {
            binding::ipset_custom_printf(self.set, None, None, Some(outfn), self as *mut _ as _);
            let session = binding::ipset_session(self.set);
            let data = binding::ipset_session_data(session);
            Session {
                session,
                data,
                set: self,
            }
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

impl<'a> Session<'a> {
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

    fn run_cmd(&mut self, cmd: binding::ipset_cmd) -> Result<(), Error> {
        unsafe {
            self.set.output.clear();
            if binding::ipset_cmd(self.session, cmd, 0) < 0 {
                Err(Error::Cmd(self.error()))
            } else {
                Ok(())
            }
        }
    }

    fn ip_cmd(&mut self, name: &str, ip: IpAddr, cmd: binding::ipset_cmd) -> Result<(), Error> {
        let name = CString::new(name).unwrap();
        self.set_data(binding::ipset_opt_IPSET_SETNAME, name.as_ptr() as _)?;

        let addr = get_caddr(ip);
        let (ip, family) = addr.as_ptr();
        self.set_data(binding::ipset_opt_IPSET_OPT_FAMILY, family)?;

        unsafe {
            let typ = binding::ipset_type_get(self.session, binding::ipset_cmd_IPSET_CMD_TEST);
            if typ.is_null() {
                return Err(Error::TypeGet(self.error()));
            }
        }

        self.set_data(binding::ipset_opt_IPSET_OPT_IP, ip)?;

        self.run_cmd(cmd)
    }

    pub fn test(&mut self, name: &str, ip: IpAddr) -> Result<bool, Error> {
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

    pub fn add(&mut self, name: &str, ip: IpAddr) -> Result<bool, Error> {
        match self.ip_cmd(name, ip, binding::ipset_cmd_IPSET_CMD_ADD) {
            Ok(_) => Ok(true),
            Err(Error::Cmd(message)) => {
                if message.contains("Element cannot be added to the set: it's already added") {
                    Ok(false)
                } else {
                    Err(Error::Cmd(message))
                }
            }
            Err(err) => Err(err),
        }
    }

    pub fn list(&mut self, name: &str) -> Result<Vec<IpAddr>, Error> {
        let name = CString::new(name).unwrap();
        self.set_data(binding::ipset_opt_IPSET_SETNAME, name.as_ptr() as _)?;

        self.run_cmd(binding::ipset_cmd_IPSET_CMD_LIST).unwrap();
        println!("{}", self.set.output[0]);
        Ok(vec![])
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
