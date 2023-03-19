use crate::compiler::functions::FunctionData;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct StructDef {
    pub field_names: Vec<String>,
    pub methods: Vec<FunctionData>,
    pub method_names: Vec<String>,
}

impl StructDef {
    pub fn new(fields: Vec<String>) -> StructDef {
        StructDef {
            field_names: fields,
            methods: Vec::new(),
            method_names: Vec::new(),
        }
    }

    pub fn get_name_map(&self) -> HashMap<String, usize> {
        let mut map = HashMap::new();
        for (i, field) in self.field_names.iter().enumerate() {
            map.insert(field.clone(), i);
        }
        let mut j = self.field_names.len();
        for (i, method) in self.methods.iter().enumerate() {
            if !method.is_static {
                map.insert(self.method_names[i].clone(), j);
                j += 1;
            }
        }

        map
    }
    pub fn has_static_method(&self, key: &String) -> bool {
        for method in self.method_names.iter() {
            if method == key {
                return true;
            }
        }
        false
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

    pub fn push_method(
        &mut self,
        key: &String,
        function: FunctionData,
        function_name: String,
    ) -> bool {
        if let Some(struc) = self.structs.get_mut(key) {
            struc.methods.push(function);
            struc.method_names.push(function_name);
            true
        } else {
            false
        }
    }

    pub fn push_definition(&mut self, name: String, struct_: StructDef) -> usize {
        self.structs.insert(name, struct_);
        self.structs.len() - 1
    }
}
