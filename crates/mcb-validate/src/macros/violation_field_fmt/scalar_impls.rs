use super::ViolationFieldFmt;

impl ViolationFieldFmt for bool {
    fn fmt_field(&self) -> String {
        self.to_string()
    }
}
