pub trait Executable {
    fn execute(&self) -> Result<(), String>;
}
