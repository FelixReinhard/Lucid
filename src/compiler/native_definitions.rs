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
        def!(self, "range", 3, 1);
        def!(self, "sleep", 4, 1);
        def!(self, "now", 5, 0);
        def!(self, "read_file", 6, 0);
        def!(self, "push", 7, 2);
        def!(self, "__string_get_at", 8, 1);
        self
    }
}
