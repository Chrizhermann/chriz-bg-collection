//! Process-environment overrides for integration tests.

use std::ffi::OsString;

/// Sets one variable in this test process and restores its previous state on drop.
///
/// Every test thread in the crate shares the process environment, so use this only for a
/// variable the crate's other tests do not depend on.
pub struct ScopedEnvVar {
    name: &'static str,
    previous: Option<OsString>,
}

impl ScopedEnvVar {
    pub fn set(name: &'static str, value: &str) -> Self {
        let previous = std::env::var_os(name);
        std::env::set_var(name, value);
        Self { name, previous }
    }

    /// Removes one variable in this test process; it is restored on drop like [`Self::set`].
    pub fn remove(name: &'static str) -> Self {
        let previous = std::env::var_os(name);
        std::env::remove_var(name);
        Self { name, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(value) => std::env::set_var(self.name, value),
            None => std::env::remove_var(self.name),
        }
    }
}
