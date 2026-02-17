#![allow(unsafe_code)]

use std::env;

pub struct EnvVarGuard {
    keys: Vec<String>,
}

impl EnvVarGuard {
    #[must_use]
    pub fn new(vars: &[(&str, &str)]) -> Self {
        for (k, v) in vars {
            unsafe {
                env::set_var(k, v);
            }
        }
        Self {
            keys: vars.iter().map(|(k, _)| (*k).to_owned()).collect(),
        }
    }

    #[must_use]
    pub fn set(key: &str, value: &str) -> Self {
        Self::new(&[(key, value)])
    }

    pub fn remove(vars: &[&str]) {
        for key in vars {
            unsafe {
                env::remove_var(key);
            }
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        for key in &self.keys {
            unsafe {
                env::remove_var(key);
            }
        }
    }
}
