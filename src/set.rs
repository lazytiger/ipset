use std::ffi::CStr;

use crate::binding;
use crate::types::Error;

pub struct IPSet {
    pub(crate) set: *mut binding::ipset,
    pub(crate) session: *mut binding::ipset_session,
}

impl IPSet {
    pub fn new() -> IPSet {
        unsafe {
            binding::ipset_load_types();
            let set = binding::ipset_init();
            let session = binding::ipset_session(set);
            IPSet { set, session }
        }
    }

    pub(crate) fn error(&self) -> (String, binding::ipset_err_type) {
        unsafe {
            let err = binding::ipset_session_report_msg(self.session);
            let err = CStr::from_ptr(err).to_string_lossy().to_string();
            let typ = binding::ipset_session_report_type(self.session);
            binding::ipset_session_report_reset(self.session);
            (err, typ)
        }
    }

    pub fn restore(&self, filename: String) -> Result<(), Error> {
        unsafe {
            let filename = std::ffi::CString::new(filename).unwrap();
            let ret = binding::ipset_session_io_normal(
                self.session,
                filename.as_ptr(),
                binding::ipset_io_type_IPSET_IO_INPUT,
            );
            if ret < 0 {
                return Err(Error::SaveRestore(self.error().0));
            }

            let file = binding::ipset_session_io_stream(
                self.session,
                binding::ipset_io_type_IPSET_IO_INPUT,
            );
            let ret = binding::ipset_parse_stream(self.set, file);
            if ret < 0 {
                Err(Error::SaveRestore(self.error().0))
            } else {
                Ok(())
            }
        }
    }
}

impl Drop for IPSet {
    fn drop(&mut self) {
        unsafe {
            binding::ipset_fini(self.set);
        }
    }
}
