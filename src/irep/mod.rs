use std::collections::HashMap;
use std::rc::Rc;

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


#[derive(Clone, Debug)]
pub struct Irept {
    pub id: usize,
    pub subt: Vec<Rc<Irept>>,
    pub named_subt: HashMap<usize, Rc<Irept>>,
    pub comments: HashMap<usize, Rc<Irept>>,
}

impl Irept {
    pub fn new(id: &str, interner: &mut StringInterner) -> Self {
        Self {
            id: interner.get_or_intern(id),
            subt: Vec::new(),
            named_subt: HashMap::new(),
            comments: HashMap::new(),
        }
    }
}

struct IreptDisplay<'a> {
    irept: &'a Irept,
    interner: &'a StringInterner,
}
use std::fmt;
impl fmt::Display for IreptDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let default = "<NOT FOUND>";
        let s = self.interner.resolve(self.irept.id).unwrap_or(&default);
        writeln!(f, "Id: {}", s)?;

        // Write subt
        writeln!(f, "Subt:")?;
        for child in &self.irept.subt {
            writeln!(
                f,
                "  {}",
                IreptDisplay {
                    irept: child,
                    interner: self.interner
                }
            )?;
        }

        writeln!(f, "Named sub:")?;
        for (key, child) in &self.irept.named_subt {
            let key_str = self.interner.resolve(*key).unwrap_or(&default);
            writeln!(
                f,
                "  {}: {}",
                key_str,
                IreptDisplay {
                    irept: child,
                    interner: self.interner
                }
            )?;
        }

        // Write comments
        writeln!(f, "Comments:")?;
        for (key, child) in &self.irept.comments {
            let key_str = self.interner.resolve(*key).unwrap_or(&default);
            writeln!(
                f,
                "  {}: {}",
                key_str,
                IreptDisplay {
                    irept: child,
                    interner: self.interner
                }
            )?;
        }

        Ok(())
    }
}

impl std::hash::Hash for Irept {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        for irep in &self.subt {
            irep.hash(state);
        }
        for (name, irep) in &self.named_subt {
            name.hash(state);
            irep.hash(state);
        }
        for (name, irep) in &self.comments {
            name.hash(state);
            irep.hash(state);
        }
    }
}

impl PartialEq for Irept {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.subt == other.subt
            && self.named_subt == other.named_subt
            && self.comments == other.comments
    }
}
impl Eq for Irept {}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test interner
    fn setup_interner() -> StringInterner {
        StringInterner::new()
    }

    // ============================================================================
    // Construction Tests
    // ============================================================================

    #[test]
    fn test_irept_new_creates_empty_structure() {
        let mut interner = setup_interner();
        let irept = Irept::new("test_id", &mut interner);

        assert_eq!(irept.subt.len(), 0);
        assert_eq!(irept.named_subt.len(), 0);
        assert_eq!(irept.comments.len(), 0);
    }

    #[test]
    fn test_irept_new_interns_id() {
        let mut interner = setup_interner();
        let irept1 = Irept::new("test_id", &mut interner);
        let irept2 = Irept::new("test_id", &mut interner);

        // Same string should result in same interned ID
        assert_eq!(irept1.id, irept2.id);
    }

    #[test]
    fn test_irept_new_different_ids() {
        let mut interner = setup_interner();
        let irept1 = Irept::new("id1", &mut interner);
        let irept2 = Irept::new("id2", &mut interner);

        assert_ne!(irept1.id, irept2.id);
    }

    // ============================================================================
    // Subt (Subtree) Storage Tests
    // ============================================================================

    #[test]
    fn test_add_single_subt() {
        let mut interner = setup_interner();
        let mut parent = Irept::new("parent", &mut interner);
        let child = Irept::new("child", &mut interner);

        parent.subt.push(Rc::new(child));

        assert_eq!(parent.subt.len(), 1);
    }

    #[test]
    fn test_add_multiple_subt() {
        let mut interner = setup_interner();
        let mut parent = Irept::new("parent", &mut interner);

        for i in 0..5 {
            let child = Irept::new(&format!("child_{}", i), &mut interner);
            parent.subt.push(Rc::new(child));
        }

        assert_eq!(parent.subt.len(), 5);
    }

    #[test]
    fn test_subt_maintains_order() {
        let mut interner = setup_interner();
        let mut parent = Irept::new("parent", &mut interner);

        let child_ids: Vec<_> = (0..3)
            .map(|i| {
                let child = Irept::new(&format!("child_{}", i), &mut interner);
                let id = child.id;
                parent.subt.push(Rc::new(child));
                id
            })
            .collect();

        for (idx, child) in parent.subt.iter().enumerate() {
            assert_eq!(child.id, child_ids[idx]);
        }
    }

    // ============================================================================
    // Named Subt Storage Tests
    // ============================================================================

    #[test]
    fn test_add_single_named_subt() {
        let mut interner = setup_interner();
        let mut parent = Irept::new("parent", &mut interner);
        let child = Irept::new("child", &mut interner);
        let name_id = interner.get_or_intern("field_name");

        parent.named_subt.insert(name_id, Rc::new(child));

        assert_eq!(parent.named_subt.len(), 1);
    }

    #[test]
    fn test_add_multiple_named_subt() {
        let mut interner = setup_interner();
        let mut parent = Irept::new("parent", &mut interner);

        for i in 0..3 {
            let child = Irept::new(&format!("child_{}", i), &mut interner);
            let name = interner.get_or_intern(&format!("field_{}", i));
            parent.named_subt.insert(name, Rc::new(child));
        }

        assert_eq!(parent.named_subt.len(), 3);
    }

    #[test]
    fn test_named_subt_overwrite() {
        let mut interner = setup_interner();
        let mut parent = Irept::new("parent", &mut interner);
        let child1 = Irept::new("child1", &mut interner);
        let child2 = Irept::new("child2", &mut interner);
        let name_id = interner.get_or_intern("field");

        let child2_id = child2.id;

        parent.named_subt.insert(name_id, Rc::new(child1));
        assert_eq!(parent.named_subt.len(), 1);

        parent.named_subt.insert(name_id, Rc::new(child2));
        assert_eq!(parent.named_subt.len(), 1);
        assert_eq!(parent.named_subt[&name_id].id, child2_id);
    }

    #[test]
    fn test_retrieve_named_subt() {
        let mut interner = setup_interner();
        let mut parent = Irept::new("parent", &mut interner);
        let child = Irept::new("child", &mut interner);
        let name_id = interner.get_or_intern("field");
        let child_id = child.id;

        parent.named_subt.insert(name_id, Rc::new(child));

        assert!(parent.named_subt.contains_key(&name_id));
        assert_eq!(parent.named_subt[&name_id].id, child_id);
    }

    // ============================================================================
    // Comments Storage Tests
    // ============================================================================

    #[test]
    fn test_add_single_comment() {
        let mut interner = setup_interner();
        let mut irept = Irept::new("node", &mut interner);
        let comment = Irept::new("comment_text", &mut interner);
        let comment_key = interner.get_or_intern("comment");

        irept.comments.insert(comment_key, Rc::new(comment));

        assert_eq!(irept.comments.len(), 1);
    }

    #[test]
    fn test_add_multiple_comments() {
        let mut interner = setup_interner();
        let mut irept = Irept::new("node", &mut interner);

        for i in 0..3 {
            let comment = Irept::new(&format!("comment_{}", i), &mut interner);
            let key = interner.get_or_intern(&format!("comment_type_{}", i));
            irept.comments.insert(key, Rc::new(comment));
        }

        assert_eq!(irept.comments.len(), 3);
    }

    #[test]
    fn test_retrieve_comment() {
        let mut interner = setup_interner();
        let mut irept = Irept::new("node", &mut interner);
        let comment = Irept::new("comment_text", &mut interner);
        let comment_id = comment.id;
        let comment_key = interner.get_or_intern("doc_comment");

        irept.comments.insert(comment_key, Rc::new(comment));

        assert_eq!(irept.comments[&comment_key].id, comment_id);
    }

    // ============================================================================
    // Equality Tests
    // ============================================================================

    #[test]
    fn test_equal_empty_irepts() {
        let mut interner = setup_interner();
        let irept1 = Irept::new("test", &mut interner);
        let irept2 = Irept::new("test", &mut interner);

        assert_eq!(irept1, irept2);
    }

    #[test]
    fn test_unequal_different_ids() {
        let mut interner = setup_interner();
        let irept1 = Irept::new("id1", &mut interner);
        let irept2 = Irept::new("id2", &mut interner);

        assert_ne!(irept1, irept2);
    }

    #[test]
    fn test_equal_with_same_subt() {
        let mut interner = setup_interner();
        let mut irept1 = Irept::new("parent", &mut interner);
        let mut irept2 = Irept::new("parent", &mut interner);

        let child1 = Irept::new("child", &mut interner);
        let child2 = Irept::new("child", &mut interner);

        irept1.subt.push(Rc::new(child1));
        irept2.subt.push(Rc::new(child2));

        assert_eq!(irept1, irept2);
    }

    #[test]
    fn test_unequal_different_subt_count() {
        let mut interner = setup_interner();
        let mut irept1 = Irept::new("parent", &mut interner);
        let mut irept2 = Irept::new("parent", &mut interner);

        let child = Irept::new("child", &mut interner);
        irept1.subt.push(Rc::new(child.clone()));

        assert_ne!(irept1, irept2);
    }

    #[test]
    fn test_equal_with_named_subt() {
        let mut interner = setup_interner();
        let mut irept1 = Irept::new("parent", &mut interner);
        let mut irept2 = Irept::new("parent", &mut interner);

        let child1 = Irept::new("child", &mut interner);
        let child2 = Irept::new("child", &mut interner);
        let name_id = interner.get_or_intern("field");

        irept1.named_subt.insert(name_id, Rc::new(child1));
        irept2.named_subt.insert(name_id, Rc::new(child2));

        assert_eq!(irept1, irept2);
    }

    #[test]
    fn test_equal_with_comments() {
        let mut interner = setup_interner();
        let mut irept1 = Irept::new("node", &mut interner);
        let mut irept2 = Irept::new("node", &mut interner);

        let comment1 = Irept::new("comment", &mut interner);
        let comment2 = Irept::new("comment", &mut interner);
        let comment_key = interner.get_or_intern("comment");

        irept1.comments.insert(comment_key, Rc::new(comment1));
        irept2.comments.insert(comment_key, Rc::new(comment2));

        assert_eq!(irept1, irept2);
    }

    // ============================================================================
    // Hashing Tests
    // ============================================================================

    #[test]
    fn test_hash_empty_irepts() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut interner = setup_interner();
        let irept1 = Irept::new("test", &mut interner);
        let irept2 = Irept::new("test", &mut interner);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        irept1.hash(&mut hasher1);
        irept2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_hash_with_subt() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut interner = setup_interner();
        let mut irept1 = Irept::new("parent", &mut interner);
        let mut irept2 = Irept::new("parent", &mut interner);

        let child1 = Irept::new("child", &mut interner);
        let child2 = Irept::new("child", &mut interner);

        irept1.subt.push(Rc::new(child1));
        irept2.subt.push(Rc::new(child2));

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        irept1.hash(&mut hasher1);
        irept2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_hash_with_named_subt() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut interner = setup_interner();
        let mut irept1 = Irept::new("parent", &mut interner);
        let mut irept2 = Irept::new("parent", &mut interner);

        let child1 = Irept::new("child", &mut interner);
        let child2 = Irept::new("child", &mut interner);
        let name_id = interner.get_or_intern("field");

        irept1.named_subt.insert(name_id, Rc::new(child1));
        irept2.named_subt.insert(name_id, Rc::new(child2));

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        irept1.hash(&mut hasher1);
        irept2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_hash_consistency() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut interner = setup_interner();
        let irept = Irept::new("test", &mut interner);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        irept.hash(&mut hasher1);
        irept.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // ============================================================================
    // Clone Tests
    // ============================================================================

    #[test]
    fn test_clone_empty_irept() {
        let mut interner = setup_interner();
        let irept = Irept::new("test", &mut interner);
        let cloned = irept.clone();

        assert_eq!(irept, cloned);
        assert_eq!(irept.id, cloned.id);
    }

    #[test]
    fn test_clone_with_children() {
        let mut interner = setup_interner();
        let mut irept = Irept::new("parent", &mut interner);
        let child = Irept::new("child", &mut interner);

        irept.subt.push(Rc::new(child));
        let cloned = irept.clone();

        assert_eq!(irept, cloned);
        assert_eq!(irept.subt.len(), cloned.subt.len());
    }

    #[test]
    fn test_clone_independence() {
        let mut interner = setup_interner();
        let irept = Irept::new("parent", &mut interner);
        let mut cloned = irept.clone();

        let child = Irept::new("child", &mut interner);
        cloned.subt.push(Rc::new(child));

        assert_eq!(irept.subt.len(), 0);
        assert_eq!(cloned.subt.len(), 1);
    }

    // ============================================================================
    // Edge Cases and Complex Scenarios
    // ============================================================================

    #[test]
    fn test_deeply_nested_structure() {
        let mut interner = setup_interner();
        let mut root = Irept::new("root", &mut interner);
        let mut current = Irept::new("level_1", &mut interner);

        for i in 2..5 {
            let child = Irept::new(&format!("level_{}", i), &mut interner);
            current.subt.push(Rc::new(child.clone()));
            current = child;
        }

        root.subt.push(Rc::new(current));
        assert_eq!(root.subt.len(), 1);
    }

    #[test]
    fn test_mixed_storage_types() {
        let mut interner = setup_interner();
        let mut irept = Irept::new("complex", &mut interner);

        // Add subt
        let subt_child = Irept::new("subt_child", &mut interner);
        irept.subt.push(Rc::new(subt_child));

        // Add named_subt
        let named_child = Irept::new("named_child", &mut interner);
        let field_name = interner.get_or_intern("field");
        irept.named_subt.insert(field_name, Rc::new(named_child));

        // Add comments
        let comment = Irept::new("comment", &mut interner);
        let comment_key = interner.get_or_intern("comment");
        irept.comments.insert(comment_key, Rc::new(comment));

        assert_eq!(irept.subt.len(), 1);
        assert_eq!(irept.named_subt.len(), 1);
        assert_eq!(irept.comments.len(), 1);
    }

    #[test]
    fn test_multiple_references_to_same_node() {
        let mut interner = setup_interner();
        let mut parent1 = Irept::new("parent1", &mut interner);
        let mut parent2 = Irept::new("parent2", &mut interner);

        let shared_child = Irept::new("shared", &mut interner);
        let shared_rc = Rc::new(shared_child);

        parent1.subt.push(shared_rc.clone());
        parent2.subt.push(shared_rc);

        assert_eq!(parent1.subt[0].id, parent2.subt[0].id);
    }

    #[test]
    fn test_empty_string_id() {
        let mut interner = setup_interner();
        let irept = Irept::new("", &mut interner);

        assert_eq!(irept.subt.len(), 0);
        assert_eq!(irept.named_subt.len(), 0);
    }

    #[test]
    fn test_special_characters_in_id() {
        let mut interner = setup_interner();
        let special_ids = vec!["id-with-dash", "id_with_underscore", "id:colon", "id.dot"];

        let irepts: Vec<_> = special_ids
            .iter()
            .map(|id| Irept::new(id, &mut interner))
            .collect();

        for irept in irepts {
            assert_ne!(irept.id, special_ids.len());
        }
    }

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
