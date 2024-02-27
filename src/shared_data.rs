use crate::{check_concern, hostcalls, Status};

/// A VM ID local atomic field. Any WASM VM in the same VM ID can read or write to any key in it's VM ID.
/// SharedData cannot cross VM IDs.
#[derive(Clone)]
pub struct SharedData<T: AsRef<str>>(T);

impl<T: AsRef<str>> SharedData<T> {
    /// Create a new/reference an existing SharedData.
    pub fn from_key(key: T) -> Self {
        Self(key)
    }

    /// Gets the value of the SharedData. Doesn't return the check-and-set (CAS) number.
    pub fn get(&self) -> Option<Vec<u8>> {
        check_concern(
            "shared-data-get",
            hostcalls::get_shared_data(self.0.as_ref()),
        )
        .and_then(|x| x.0)
    }

    /// Gets the value and the check-and-set (CAS) number of the SharedData. CAS is `None` when the value has never been set before.
    pub fn get_with_cas(&self) -> (Option<Vec<u8>>, Option<u32>) {
        check_concern(
            "shared-data-get-cas",
            hostcalls::get_shared_data(self.0.as_ref()),
        )
        .unwrap_or_default()
    }

    /// Unconditionally sets the value of this SharedData.
    pub fn set(&self, value: impl AsRef<[u8]>) {
        check_concern(
            "shared-data-set-casless",
            hostcalls::set_shared_data(self.0.as_ref(), Some(value.as_ref()), None),
        );
    }

    /// Sets the value of this SharedData only when the given `cas` number matches the one returned by a previous `get_with_cas`.
    pub fn set_with_cas(&self, value: impl AsRef<[u8]>, cas: u32) -> bool {
        match hostcalls::set_shared_data(self.0.as_ref(), Some(value.as_ref()), Some(cas)) {
            Ok(()) => true,
            Err(Status::CasMismatch) => false,
            Err(e) => {
                check_concern::<()>("shared-data-set-cas", Err(e));
                false
            }
        }
    }
}
