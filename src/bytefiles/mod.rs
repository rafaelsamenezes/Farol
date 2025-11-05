use crate::irep::Irept;
use crate::irep::StringInterner;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;
use std::str;

pub struct ByteReader {
    file: Vec<u8>,
    pointer: usize,
    pub irep_container: HashMap<u32, Rc<Irept>>,
    string_ref_container: HashMap<u32, usize>,
    pub string_interner: StringInterner,
}

impl From<Vec<u8>> for ByteReader {
    fn from(data: Vec<u8>) -> Self {
        ByteReader {
            file: data,
            pointer: 0,
            irep_container: HashMap::new(),
            string_ref_container: HashMap::new(),
            string_interner: StringInterner::new(),
        }
    }
}

impl ByteReader {
    pub fn read_file(path: &str) -> Self {
        let byte_content = fs::read(path).expect(format!("Could not read file {}", path).as_str());
        ByteReader::from(byte_content)
    }

    fn peek(&self) -> u8 {
        self.file[self.pointer]
    }

    fn get(&mut self) -> u8 {
        let value = self.file[self.pointer];
        self.pointer += 1;
        value
    }

    // Reference parsing. First try the cache, if not available then parse the irep
    pub fn read_esbmc_reference(&mut self) -> Rc<Irept> {
        let id = self.read_esbmc_word();
        if self.irep_container.contains_key(&id) {
            return self.irep_container.get(&id).unwrap().clone();
        }

        let irep_id = self.read_esbmc_string_ref();
        // Sub-expression
        let mut irep_sub: Vec<Rc<Irept>> = Vec::new();
        while self.peek() == b'S' {
            self.pointer += 1;
            let sub = self.read_esbmc_reference();
            irep_sub.push(sub);
        }

        // Named sub
        let mut named_sub: HashMap<usize, Rc<Irept>> = HashMap::new();
        while self.peek() == b'N' {
            self.pointer += 1;
            let named_id = self.read_esbmc_string_ref();
            // TODO: assert named_id[0] != '#'
            named_sub.insert(named_id, self.read_esbmc_reference());
        }

        // Comment?
        let mut comments_sub: HashMap<usize, Rc<Irept>> = HashMap::new();
        while self.peek() == b'C' {
            self.pointer += 1;
            let named_id = self.read_esbmc_string_ref();
            // TODO: assert named_id[0] == '#'
            comments_sub.insert(named_id, self.read_esbmc_reference());
        }

        let end_value = self.get();
        if end_value != 0 {
            panic!("Irep not terminated.");
        }

        let result = Irept {
            id: irep_id,
            subt: irep_sub,
            named_subt: named_sub,
            comments: comments_sub,
        };

        self.irep_container.insert(id, Rc::new(result)).unwrap()
    }

    // String parsing.
    pub fn read_esbmc_string(&mut self) -> String {
        let mut bytes = Vec::<u8>::new();
        while self.peek() != 0 {
            let c = self.get();
            if c == b'\\' {
                bytes.push(self.get());
            } else {
                bytes.push(c);
            }
        }
        self.pointer += 1;
        let value = String::from_utf8_lossy(&bytes).to_string();
        value
    }

    // String reference parsing. Similar than the irep one
    pub fn read_esbmc_string_ref(&mut self) -> usize {
        let id = self.read_esbmc_word();

        if self.string_ref_container.contains_key(&id) {
            return self.string_ref_container.get(&id).unwrap().clone();
        }

        let value = self.read_esbmc_string();
        let interner_id = self.string_interner.get_or_intern(&value);
        self.string_ref_container.insert(id, interner_id);
        interner_id
    }

    // Word reading (as u32)
    pub fn read_esbmc_word(&mut self) -> u32 {
        let raw_bytes: &[u8; 4] = self.file[self.pointer..self.pointer + 4]
            .try_into()
            .expect("Slice should be of length 4");
        self.pointer += 4;

        // ESBMC generates this in BE form
        u32::from_be_bytes(*raw_bytes)
    }

    pub fn check_esbmc_header(&mut self) -> Result<(), String> {
        let header: &[u8; 3] = self.file[0..3]
            .try_into()
            .expect("GBF does not contain header");

        let gbf: [u8; 3] = [b'G', b'B', b'F'];
        if *header != gbf {
            return Err(format!(
                "Invalid ESBMC header. Found: {}{}{}",
                header[0], header[1], header[2]
            ));
        }
        self.pointer = 3;
        Ok(())
    }

    pub fn check_esbmc_version(&mut self) -> Result<(), String> {
        let version = self.read_esbmc_word();
        if version != 1 {
            return Err(format!("Invalid ESBMC version. Found {}", version));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    // ===== Byte-level operations =====

    #[test]
    fn test_peek_and_get_sequence() {
        let data = vec![1, 2, 3, 4, 5];
        let mut reader = ByteReader::from(data);

        assert_eq!(reader.peek(), 1);
        assert_eq!(reader.peek(), 1); // peek doesn't advance
        assert_eq!(reader.get(), 1); // get advances
        assert_eq!(reader.peek(), 2);
        assert_eq!(reader.get(), 2);
    }

    // ===== Header validation =====

    #[test]
    fn test_check_esbmc_header_valid() {
        let mut data = vec![b'G', b'B', b'F'];
        data.extend_from_slice(&[0u8; 4]); // Padding for next read
        let mut reader = ByteReader::from(data);

        assert!(reader.check_esbmc_header().is_ok());
        assert_eq!(reader.pointer, 3);
    }

    #[test]
    fn test_check_esbmc_header_invalid_first_byte() {
        let data = vec![b'X', b'B', b'F', 0, 0, 0, 0];
        let mut reader = ByteReader::from(data);

        let result = reader.check_esbmc_header();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid ESBMC header"));
    }

    #[test]
    fn test_check_esbmc_header_invalid_second_byte() {
        let data = vec![b'G', b'X', b'F', 0, 0, 0, 0];
        let mut reader = ByteReader::from(data);

        assert!(reader.check_esbmc_header().is_err());
    }

    #[test]
    fn test_check_esbmc_header_invalid_third_byte() {
        let data = vec![b'G', b'B', b'X', 0, 0, 0, 0];
        let mut reader = ByteReader::from(data);

        assert!(reader.check_esbmc_header().is_err());
    }

    // ===== Version validation =====

    #[test]
    fn test_check_esbmc_version_valid() {
        // Version 1 in big-endian: [0, 0, 0, 1]
        let data = vec![0, 0, 0, 1];
        let mut reader = ByteReader::from(data);

        assert!(reader.check_esbmc_version().is_ok());
    }

    #[test]
    fn test_check_esbmc_version_invalid_too_high() {
        // Version 2 in big-endian: [0, 0, 0, 2]
        let data = vec![0, 0, 0, 2];
        let mut reader = ByteReader::from(data);

        let result = reader.check_esbmc_version();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid ESBMC version"));
    }

    #[test]
    fn test_check_esbmc_version_invalid_zero() {
        let data = vec![0, 0, 0, 0];
        let mut reader = ByteReader::from(data);

        assert!(reader.check_esbmc_version().is_err());
    }

    // ===== Word reading (u32) =====

    #[test]
    fn test_read_esbmc_word_big_endian() {
        // 0x12345678 in big-endian
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let mut reader = ByteReader::from(data);

        let word = reader.read_esbmc_word();
        assert_eq!(word, 0x12345678);
        assert_eq!(reader.pointer, 4);
    }

    #[test]
    fn test_read_esbmc_word_zero() {
        let data = vec![0, 0, 0, 0];
        let mut reader = ByteReader::from(data);

        assert_eq!(reader.read_esbmc_word(), 0);
    }

    #[test]
    fn test_read_esbmc_word_max() {
        // 0xFFFFFFFF
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let mut reader = ByteReader::from(data);

        assert_eq!(reader.read_esbmc_word(), 0xFFFFFFFF);
    }

    #[test]
    fn test_read_esbmc_word_sequence() {
        let data = vec![
            0, 0, 0, 1, // First word: 1
            0, 0, 0, 2, // Second word: 2
            0, 0, 0, 3, // Third word: 3
        ];
        let mut reader = ByteReader::from(data);

        assert_eq!(reader.read_esbmc_word(), 1);
        assert_eq!(reader.read_esbmc_word(), 2);
        assert_eq!(reader.read_esbmc_word(), 3);
    }

    // ===== String reading =====

    #[test]
    fn test_read_esbmc_string_simple() {
        let mut data = b"hello\0".to_vec();
        let mut reader = ByteReader::from(data);

        let s = reader.read_esbmc_string();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_read_esbmc_string_empty() {
        let data = vec![0];
        let mut reader = ByteReader::from(data);

        let s = reader.read_esbmc_string();
        assert_eq!(s, "");
    }

    #[test]
    fn test_read_esbmc_string_with_numbers() {
        let mut data = b"test123\0".to_vec();
        let mut reader = ByteReader::from(data);

        assert_eq!(reader.read_esbmc_string(), "test123");
    }

    // ===== Initialization =====

    #[test]
    fn test_from_vec_u8() {
        let data = vec![1, 2, 3, 4];
        let reader = ByteReader::from(data);

        assert_eq!(reader.pointer, 0);
        assert_eq!(reader.irep_container.len(), 0);
        assert_eq!(reader.string_ref_container.len(), 0);
    }

    #[test]
    fn test_read_file_not_found() {
        let result = std::panic::catch_unwind(|| {
            ByteReader::read_file("nonexistent/file.esbmc");
        });
        assert!(result.is_err());
    }

    // ===== Container operations =====

    #[test]
    fn test_irep_container_caching() {
        let data = vec![0, 0, 0, 1];
        let mut reader = ByteReader::from(data);

        // Manually insert into container for testing
        let test_irep = Irept {
            id: 0,
            subt: Vec::new(),
            named_subt: HashMap::new(),
            comments: HashMap::new(),
        };
        reader.irep_container.insert(123, Rc::new(test_irep));

        assert!(reader.irep_container.contains_key(&123));
        assert_eq!(reader.irep_container.len(), 1);
    }

    #[test]
    fn test_string_interner() {
        let data = vec![];
        let mut reader = ByteReader::from(data);

        let id1 = reader.string_interner.get_or_intern("hello");
        let id2 = reader.string_interner.get_or_intern("hello");
        let id3 = reader.string_interner.get_or_intern("world");

        assert_eq!(id1, id2); // Same string should have same id
        assert_ne!(id1, id3); // Different strings should have different ids
    }
}
