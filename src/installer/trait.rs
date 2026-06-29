pub trait PackageManager {
    fn install(&self, packages: &[String]) -> Vec<Result<(), String>>;
    fn name(&self) -> &'static str;
    fn is_available() -> bool
    where
        Self: Sized;
}
