use std::path::PathBuf;

use super::ViolationFieldFmt;

impl ViolationFieldFmt for PathBuf {
    fn fmt_field(&self) -> String {
        self.display().to_string()
    }
}

impl ViolationFieldFmt for String {
    fn fmt_field(&self) -> String {
        self.clone()
    }
}

impl ViolationFieldFmt for &str {
    fn fmt_field(&self) -> String {
        (*self).to_owned()
    }
}

impl ViolationFieldFmt for usize {
    fn fmt_field(&self) -> String {
        self.to_string()
    }
}

impl ViolationFieldFmt for u32 {
    fn fmt_field(&self) -> String {
        self.to_string()
    }
}

impl ViolationFieldFmt for i32 {
    fn fmt_field(&self) -> String {
        self.to_string()
    }
}

impl ViolationFieldFmt for i64 {
    fn fmt_field(&self) -> String {
        self.to_string()
    }
}
