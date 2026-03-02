#[derive(Debug)]
pub struct VmOptions {
    pub raw_errors: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for VmOptions {
    fn default() -> Self {
        Self { raw_errors: false }
    }
}
