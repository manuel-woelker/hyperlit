use std::collections::HashSet;

#[derive(Default)]
pub struct IdGenerator {
    ids: HashSet<String>,
}

impl IdGenerator {
    pub fn id_from(&mut self, string: &str) -> String {
        let raw_candidate = string.to_string();
        let mut candidate = string.to_string();
        if self.ids.insert(candidate) {
            return raw_candidate;
        }
        let max_tries = 100;
        for i in 2..max_tries {
            candidate = format!("{} {}", raw_candidate, i);
            if self.ids.insert(candidate.clone()) {
                return candidate;
            }
        }
        panic!("Could not generate unique id from string after {max_tries} tries: '{string}'");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_from() {
        let mut id_gen = IdGenerator::default();
        assert_eq!(id_gen.id_from("Hello World"), "Hello World");
        assert_eq!(id_gen.id_from("Hello World"), "Hello World 2");
        assert_eq!(id_gen.id_from("Hello World 4"), "Hello World 4");
        assert_eq!(id_gen.id_from("Hello World"), "Hello World 3");
        assert_eq!(id_gen.id_from("Hello World"), "Hello World 5");
    }
}
