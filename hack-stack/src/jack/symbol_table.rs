use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SymbolKind {
    Var,
    Arg,
    Static,
    Field,
    This,
}

impl SymbolKind {
    pub fn segment_name(&self) -> &'static str {
        match self {
            SymbolKind::Var => "local",
            SymbolKind::Arg => "argument",
            SymbolKind::Static => "static",
            SymbolKind::Field => "this",
            SymbolKind::This => "pointer",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SymbolTableEntry<'a> {
    pub kind: SymbolKind,
    pub ty: &'a str,
    pub index: u16,
}

pub struct SymbolTable<'a> {
    table: HashMap<&'a str, SymbolTableEntry<'a>>,
}

impl<'a> SymbolTable<'a> {
    pub fn new() -> SymbolTable<'a> {
        SymbolTable {
            table: HashMap::new(),
        }
    }

    pub fn add(&mut self, kind: SymbolKind, ty: &'a str, name: &'a str) {
        let index = self.num_entries(kind);
        self.table
            .insert(name, SymbolTableEntry { index, kind, ty });
    }

    pub fn get(&self, name: &str) -> Option<&SymbolTableEntry<'a>> {
        self.table.get(name)
    }

    pub fn reset(&mut self) {
        self.table.clear();
    }

    pub fn num_entries(&self, kind: SymbolKind) -> u16 {
        self.table.iter().filter(|(_, e)| e.kind == kind).count() as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut t = SymbolTable::new();

        assert!(t.get("a").is_none());

        t.add(SymbolKind::Arg, "int", "a");
        let entry = t.get("a");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().kind, SymbolKind::Arg);
        assert_eq!(entry.unwrap().ty, "int");
        assert_eq!(entry.unwrap().index, 0);

        t.add(SymbolKind::Var, "int", "b");
        assert_eq!(t.get("b").unwrap().index, 0);

        t.add(SymbolKind::Arg, "int", "c");
        assert_eq!(t.get("c").unwrap().index, 1);
    }
}
