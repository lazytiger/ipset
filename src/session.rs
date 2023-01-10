use std::ffi::{CStr, CString};
use std::marker::PhantomData;

use crate::binding;
use crate::types::{Error, Parse, SetData, SetType, ToCString, TypeName};

/// output function required by libipset to get list output.
#[no_mangle]
pub unsafe extern "C" fn outfn(
    _session: *mut binding::ipset_session,
    p: *mut ::std::os::raw::c_void,
    _fmt: *const ::std::os::raw::c_char,
    mut args: ...
) -> ::std::os::raw::c_int {
    let raw = args.arg::<*const std::ffi::c_char>();
    let data = CStr::from_ptr(raw);
    let len = data.to_bytes().len();
    if len == 0 {
        return 0;
    }
    let output = (p as *mut Vec<String>).as_mut().unwrap();
    output.push(data.to_string_lossy().to_string());
    0
}

/// This is the main entry for all the operation. I just ignore the ipset struct
/// because all the operation are performed by session. The `output` field is used
/// for collecting data for commands like `list`. It is a field for safety.
pub struct Session<T: SetType> {
    name: CString,
    session: *mut binding::ipset_session,
    data: *mut binding::ipset_data,
    set: *mut binding::ipset,
    output: Vec<String>,
    _phantom: PhantomData<T>,
}

impl<T: SetType> Session<T> {
    /// load ipset types, initialize ipset, prepare session and data.
    pub fn new(name: String) -> Session<T> {
        unsafe {
            binding::ipset_load_types();
            let set = binding::ipset_init();
            let session = binding::ipset_session(set);
            let data = binding::ipset_session_data(session);
            Self {
                session,
                data,
                set,
                name: CString::new(name).unwrap(),
                output: Default::default(),
                _phantom: Default::default(),
            }
        }
    }

    pub(crate) fn set_data(
        &self,
        opt: binding::ipset_opt,
        value: *const std::ffi::c_void,
    ) -> Result<(), Error> {
        unsafe {
            if binding::ipset_data_set(self.data, opt, value) < 0 {
                let (message, error) = self.error();
                Err(Error::DataSet(message, error))
            } else {
                Ok(())
            }
        }
    }

    /// Get report message and whether the message is error.
    fn error(&self) -> (String, bool) {
        unsafe {
            let err = binding::ipset_session_report_msg(self.session);
            let err = CStr::from_ptr(err).to_string_lossy().to_string();
            let typ = binding::ipset_session_report_type(self.session);
            binding::ipset_session_report_reset(self.session);
            (err, typ == binding::ipset_err_type_IPSET_ERROR)
        }
    }

    fn run_cmd(&mut self, cmd: binding::ipset_cmd) -> Result<(), Error> {
        unsafe {
            self.output.clear();
            if binding::ipset_cmd(self.session, cmd, 0) < 0 {
                let (message, error) = self.error();
                Err(Error::Cmd(message, error))
            } else {
                Ok(())
            }
        }
    }

    /// Wrapper for ipset_type_get, set OPT_TYPE for the cmd
    fn get_type(&self, cmd: binding::ipset_cmd) -> Result<(), Error> {
        unsafe {
            let typ = binding::ipset_type_get(self.session, cmd);
            if typ.is_null() {
                let (message, error) = self.error();
                Err(Error::TypeGet(message, error))
            } else {
                Ok(())
            }
        }
    }

    /// Run all the ip related commands, like add/del/test
    fn data_cmd(&mut self, data: T::DataType, cmd: binding::ipset_cmd) -> Result<(), Error> {
        self.set_data(binding::ipset_opt_IPSET_SETNAME, self.name.as_ptr() as _)?;
        data.set_data(self)?;
        self.get_type(cmd)?;
        self.run_cmd(cmd)
    }

    /// Test if `ip` is in ipset `name`
    pub fn test(&mut self, data: impl Into<T::DataType>) -> Result<bool, Error> {
        self.data_cmd(data.into(), binding::ipset_cmd_IPSET_CMD_TEST)
            .map(|_| true)
            .or_else(|err| {
                if err.cmd_contains(" is NOT in set test") {
                    Ok(false)
                } else {
                    Err(err)
                }
            })
    }

    /// Add `ip` into ipset `name`
    pub fn add(&mut self, data: impl Into<T::DataType>) -> Result<bool, Error> {
        self.data_cmd(data.into(), binding::ipset_cmd_IPSET_CMD_ADD)
            .map(|_| true)
            .or_else(|err| {
                if err.cmd_contains("Element cannot be added to the set: it's already added") {
                    Ok(false)
                } else {
                    Err(err)
                }
            })
    }

    /// Delete `ip` from ipset `name`
    pub fn del(&mut self, ip: impl Into<T::DataType>) -> Result<bool, Error> {
        self.data_cmd(ip.into(), binding::ipset_cmd_IPSET_CMD_DEL)
            .map(|_| true)
            .or_else(|err| {
                if err.cmd_contains("Element cannot be deleted from the set: it's not added") {
                    Ok(false)
                } else {
                    Err(err)
                }
            })
    }

    /// Run all the name only related command like flush/list/destroy
    fn name_cmd(&mut self, cmd: binding::ipset_cmd) -> Result<bool, Error> {
        self.set_data(binding::ipset_opt_IPSET_SETNAME, self.name.as_ptr() as _)?;

        self.run_cmd(cmd).map(|_| true).or_else(|err| {
            if let Error::Cmd(_, false) = err {
                Ok(false)
            } else {
                Err(err)
            }
        })
    }

    /// List all the ips in ipset `name`
    pub fn list(&mut self) -> Result<Vec<T::DataType>, Error> {
        unsafe {
            binding::ipset_custom_printf(
                self.set,
                None,
                None,
                Some(outfn),
                &mut self.output as *mut _ as _,
            );
        }
        self.name_cmd(binding::ipset_cmd_IPSET_CMD_LIST)?;
        if let Some(line) = self.output.get(0) {
            let ips: Vec<_> = line
                .split("\n")
                .skip(8)
                .filter_map(|line| {
                    let mut data = T::DataType::default();
                    if data.parse(line).is_err() {
                        None
                    } else {
                        Some(data)
                    }
                })
                .collect();
            Ok(ips)
        } else {
            Ok(vec![])
        }
    }

    /// Clear all the content in ipset `name`
    pub fn flush(&mut self) -> Result<bool, Error> {
        self.name_cmd(binding::ipset_cmd_IPSET_CMD_FLUSH)
    }

    /// Destroy the ipset `name`
    pub fn destroy(&mut self) -> Result<bool, Error> {
        self.name_cmd(binding::ipset_cmd_IPSET_CMD_DESTROY)
    }

    /// Create a ipset `name` with type `typename` and more configuration using `f`
    pub fn create<F>(&mut self, f: F) -> Result<bool, Error>
    where
        F: Fn(CreateBuilder<T>) -> Result<(), Error>,
        T: SetType,
        T::Method: TypeName,
        T::DataType: TypeName,
    {
        unsafe {
            binding::ipset_data_reset(self.data);
            let typename = T::to_cstring();
            self.set_data(
                binding::ipset_opt_IPSET_OPT_TYPENAME,
                typename.as_ptr() as _,
            )?;
            self.get_type(binding::ipset_cmd_IPSET_CMD_CREATE)?;
        }
        let builder = CreateBuilder { session: self };
        f(builder)?;
        self.name_cmd(binding::ipset_cmd_IPSET_CMD_CREATE)
    }
}

/// Helper for creating a ipset
pub struct CreateBuilder<'a, T: SetType> {
    session: &'a Session<T>,
}

impl<'a, T: SetType> CreateBuilder<'a, T> {
    /// All  set types supports the optional timeout parameter when creating a set and adding entries.
    /// The value of the timeout parameter for the create command means the default timeout value (in seconds) for new entries.
    /// If a set is created with timeout support, then the same timeout option can  be  used  to  specify  non-default
    /// timeout  values when adding entries. Zero timeout value means the entry is added permanent to the set.
    pub fn with_timeout(self, timeout: u32) -> Result<Self, Error> {
        self.session.set_data(
            binding::ipset_opt_IPSET_OPT_TIMEOUT,
            &timeout as *const _ as _,
        )?;
        Ok(self)
    }

    /// All  set types support the optional counters option when creating a set.
    ///  If the option is specified then the set is created with packet and byte counters per element support.
    /// The packet and byte counters are initialized to zero when the elements are (re-)added to the set,
    /// unless the packet and byte counter values are  explicitly specified by the packets and bytes options.
    pub fn with_counters(self, packets: Option<u64>, bytes: Option<u64>) -> Result<Self, Error> {
        self.session
            .set_data(binding::ipset_opt_IPSET_OPT_COUNTERS, &1 as *const _ as _)?;
        if let Some(packets) = packets {
            self.session.set_data(
                binding::ipset_opt_IPSET_OPT_PACKETS,
                &packets as *const _ as _,
            )?;
        }
        if let Some(bytes) = bytes {
            self.session
                .set_data(binding::ipset_opt_IPSET_OPT_BYTES, &bytes as *const _ as _)?;
        }
        Ok(self)
    }

    /// This parameter is valid for the create command of all hash type sets.  
    /// It defines the initial hash size for the set, default is 1024.
    /// The  hash  size  must  be  a power of two, the kernel automatically rounds up non power of two hash sizes to the first correct value.
    pub fn with_hash_size(self, size: u32) -> Result<Self, Error> {
        self.session.set_data(
            binding::ipset_opt_IPSET_OPT_HASHSIZE,
            &size as *const _ as _,
        )?;
        Ok(self)
    }

    /// This  parameter  is  valid  for  the  create command of all hash type sets.  
    /// It does define the maximal number of elements which can be stored in the set, default 65536
    pub fn with_max_elem(self, max: u32) -> Result<Self, Error> {
        self.session
            .set_data(binding::ipset_opt_IPSET_OPT_MAXELEM, &max as *const _ as _)?;
        Ok(self)
    }

    /// This parameter is valid for the create command of all hash type sets except for hash:mac.  
    /// It defines the protocol family of the IP addresses to be stored in the set. The default is inet, i.e IPv4.
    pub fn with_ipv6(self, ipv6: bool) -> Result<Self, Error> {
        let value = if ipv6 {
            binding::NFPROTO_IPV6
        } else {
            binding::NFPROTO_IPV4
        };
        self.session
            .set_data(binding::ipset_opt_IPSET_OPT_FAMILY, &value as *const _ as _)?;
        Ok(self)
    }

    /// last call to end the invocation.
    pub fn build(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<T: SetType> Drop for Session<T> {
    fn drop(&mut self) {
        unsafe {
            if !self.set.is_null() {
                binding::ipset_fini(self.set);
            }
        }
    }
}

unsafe impl<T: SetType> Sync for Session<T> {}

unsafe impl<T: SetType> Send for Session<T> {}
