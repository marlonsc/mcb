use std::path::PathBuf;

use super::ViolationFieldFmt;

impl ViolationFieldFmt for Vec<String> {
    fn fmt_field(&self) -> String {
        self.join(", ")
    }
}

impl ViolationFieldFmt for Vec<PathBuf> {
    fn fmt_field(&self) -> String {
        self.iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl ViolationFieldFmt for Vec<(PathBuf, usize)> {
    fn fmt_field(&self) -> String {
        self.iter()
            .map(|(p, n)| format!("{}:{}", p.display(), n))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl ViolationFieldFmt for Vec<(PathBuf, usize, String)> {
    fn fmt_field(&self) -> String {
        self.iter()
            .map(|(p, n, s)| format!("{}:{}:{}", p.display(), n, s))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl ViolationFieldFmt for Vec<(PathBuf, String)> {
    fn fmt_field(&self) -> String {
        self.iter()
            .map(|(p, s)| format!("{}:{}", p.display(), s))
            .collect::<Vec<_>>()
            .join(", ")
    }
}
