use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct UniqueName {
    history: HashMap<String, usize>,
    anonymous: usize
}

impl UniqueName {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_name(&mut self, base: &str) -> String {
        self.anonymous += 1;
        if self.history.contains_key(base) {
            let n = self.history.get_mut(base).unwrap();
            *n += 1;
            format!("{}.{}", base, n)
        } else {
            self.history.insert(base.to_string(), 0);
            format!("{}.0", base)
        }
    }

    pub fn next_anonymous_name(&mut self) -> String {
        self.anonymous += 1;
        format!("{}", self.anonymous)
    }
}