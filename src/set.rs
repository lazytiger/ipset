use std::ffi::CStr;

use crate::binding;
use crate::types::Error;

/// Wrapper for ipset instance in c.
pub struct IPSet {
    pub(crate) set: *mut binding::ipset,
    pub(crate) session: *mut binding::ipset_session,
}

impl IPSet {
    /// Create a new IPSet instance.
    pub fn new() -> IPSet {
        unsafe {
            binding::ipset_load_types();
            let set = binding::ipset_init();
            let session = binding::ipset_session(set);
            IPSet { set, session }
        }
    }

    /// get the error message and type.
    pub(crate) fn error(&self) -> (String, binding::ipset_err_type) {
        unsafe {
            let err = binding::ipset_session_report_msg(self.session);
            let err = CStr::from_ptr(err).to_string_lossy().to_string();
            let typ = binding::ipset_session_report_type(self.session);
            binding::ipset_session_report_reset(self.session);
            (err, typ)
        }
    }

    /// Restore a set from a file.
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
            if file.is_null() {
                return Err(Error::SaveRestore(self.error().0));
            }
            let mut context = std::mem::MaybeUninit::<binding::ipset_restore_context>::zeroed();
            let ret =
                binding::ipset_restore_with_error_capture(self.set, file, context.as_mut_ptr());
            let context = context.assume_init();
            let close_ret = binding::ipset_session_io_close(
                self.session,
                binding::ipset_io_type_IPSET_IO_INPUT,
            );

            if ret < 0 || context.failed != 0 {
                let message = context
                    .message
                    .iter()
                    .take_while(|character| **character != 0)
                    .map(|character| *character as u8)
                    .collect::<Vec<_>>();
                let message = String::from_utf8_lossy(&message).trim().to_string();
                return Err(Error::SaveRestore(if message.is_empty() {
                    self.error().0
                } else {
                    message
                }));
            }

            if close_ret < 0 {
                return Err(Error::SaveRestore(self.error().0));
            }

            Ok(())
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn invalid_restore_input_returns_an_error() {
        let path =
            std::env::temp_dir().join(format!("ipset-invalid-restore-{}.txt", std::process::id()));
        fs::write(&path, "add my-set\n").unwrap();

        let result = IPSet::new().restore(path.to_string_lossy().into_owned());
        let _ = fs::remove_file(path);

        match result {
            Err(Error::SaveRestore(message)) => {
                assert!(message.contains("Missing second mandatory argument"));
            }
            other => panic!("expected SaveRestore error, got {other:?}"),
        }
    }
}
