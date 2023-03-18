use std::collections::HashMap;

#[derive(Clone)]
pub struct StructDef {
    pub field_names: Vec<String>,
}

impl StructDef {
    pub fn new(fields: Vec<String>) -> StructDef {
        StructDef{field_names: fields}
    }

    pub fn get_name_map(&self) -> HashMap<String, usize> {
        let mut map = HashMap::new();
        
        for (i, field) in self.field_names.iter().enumerate() {
            map.insert(field.clone(), i);
        }
        map
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

    pub fn get(&self, key: &String) -> Option<StructDef> {
        if let Some(v) = self.structs.get(key) {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn push_definition(&mut self, name: String, struct_: StructDef) -> usize {
        self.structs.insert(name, struct_);
        self.structs.len() - 1
    }

}
