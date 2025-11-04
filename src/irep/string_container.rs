use std::collections::HashMap;

pub struct StringInterner {
    map: HashMap<Box<str>, usize>,
    strings: Vec<Box<str>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            strings: Vec::new(),
        }
    }

    pub fn get_or_intern(&mut self, s: &str) -> usize {
        if let Some(&index) = self.map.get(s) {
            index
        } else {
            let boxed: Box<str> = s.into();
            let index = self.strings.len();
            self.strings.push(boxed.clone());
            self.map.insert(boxed, index);
            index
        }
    }

    pub fn resolve(&self, index: usize) -> Option<&str> {
        self.strings.get(index).map(|b| &**b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_interner_is_empty() {
        let interner = StringInterner::new();
        assert!(interner.strings.is_empty());
        assert!(interner.map.is_empty());
    }

    #[test]
    fn test_intern_new_string() {
        let mut interner = StringInterner::new();
        let index = interner.get_or_intern("hello");
        assert_eq!(index, 0);
        assert_eq!(interner.strings.len(), 1);
        assert_eq!(interner.resolve(index), Some("hello"));
    }

    #[test]
    fn test_intern_existing_string() {
        let mut interner = StringInterner::new();
        let index1 = interner.get_or_intern("hello");
        let index2 = interner.get_or_intern("hello");
        assert_eq!(index1, index2);
        assert_eq!(interner.strings.len(), 1);
    }

    #[test]
    fn test_resolve_invalid_index() {
        let interner = StringInterner::new();
        assert_eq!(interner.resolve(0), None);
        assert_eq!(interner.resolve(10), None);
    }

    #[test]
    fn test_multiple_strings() {
        let mut interner = StringInterner::new();
        let index1 = interner.get_or_intern("hello");
        let index2 = interner.get_or_intern("world");
        assert_ne!(index1, index2);
        assert_eq!(interner.resolve(index1), Some("hello"));
        assert_eq!(interner.resolve(index2), Some("world"));
    }
}
