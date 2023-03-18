use std::collections::HashMap;

pub struct StructDef {
    pub field_names: Vec<String>,
}

impl StructDef {
    pub fn new(fields: Vec<String>) -> StructDef {
        StructDef{field_names: fields}
    }
}

pub struct StructTable {
    structs: HashMap<String, StructDef>,
}

impl StructTable {
    pub fn new() -> StructTable {
        StructTable {
            structs: HashMap::new(),
        }
    }

    pub fn get(&self, key: &String) -> Option<&StructDef> {
        self.structs.get(key)
    }

    pub fn push_definition(&mut self, name: String, struct_: StructDef) -> usize {
        self.structs.insert(name, struct_);
        self.structs.len() - 1
    }
}
