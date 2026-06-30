pub trait PackageManager {
    fn install(&self, packages: &[String]) -> Vec<Result<(), String>>;
    fn remove(&self, packages: &[String]) -> Vec<Result<(), String>>;
    fn is_installed(&self, package: &str) -> bool;
    fn name(&self) -> &'static str;
    fn is_available() -> bool
    where
        Self: Sized;
}
