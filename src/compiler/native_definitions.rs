use crate::compiler::core::Compiler;

macro_rules! def {
    ($s:expr, $name:literal, $id:expr, $args:expr) => {
        $s.functions.add_native($name.to_string(), $id, $args)
    } 
}

impl Compiler {
    // defines all native functions
    pub fn define_natives(mut self) -> Compiler {
        def!(self, "print", 0, 1);
        def!(self, "read", 1, 0);
        def!(self, "len", 2, 1);
        self
    }
}
