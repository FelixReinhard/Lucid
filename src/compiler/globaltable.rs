use std::collections::HashMap;

pub struct GlobalTable {
    globals: HashMap<String, usize>,
    top: usize,
}

impl GlobalTable {
    pub fn new() -> GlobalTable {
        GlobalTable{globals: HashMap::new(), top: 0}
    }

    pub fn get(&self, key: &String) -> Option<usize> {
        match self.globals.get(key) {
            Some(p) => Some(*p),
            None => None,
        }
    }

    pub fn put(&mut self, key: String) -> usize {
        self.globals.insert(key, self.top);
        self.top += 1;
        self.top - 1
    }


}
