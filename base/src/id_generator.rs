use std::collections::HashSet;

#[derive(Default)]
pub struct IdGenerator {
    ids: HashSet<String>,
}

impl IdGenerator {
    pub fn id_from(&mut self, string: &str) -> String {
        let raw_candidate = string
            .replace(|c: char| !c.is_alphanumeric(), "_")
            .to_lowercase();
        let mut candidate = raw_candidate.clone();
        if self.ids.insert(candidate) {
            return raw_candidate;
        }
        let max_tries = 100;
        for i in 2..max_tries {
            candidate = format!("{}_{}", raw_candidate, i);
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
        assert_eq!(id_gen.id_from("Hello World"), "hello_world");
        assert_eq!(id_gen.id_from("Hello World"), "hello_world_2");
        assert_eq!(id_gen.id_from("Hello World 4"), "hello_world_4");
        assert_eq!(id_gen.id_from("Hello World"), "hello_world_3");
        assert_eq!(id_gen.id_from("Hello World"), "hello_world_5");
    }
}
