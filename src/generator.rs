struct GeneratorRule<T> {
    pub options: Vec<T>,
}

impl<T: Clone> GeneratorRule<T> {
    pub fn new(options: &[T]) -> Self {
        GeneratorRule {
            options: options.into(),
        }
    }

    pub fn Generate<R: FnMut(usize) -> usize>(&self, mut rng: R) -> Option<&T> {
        let selection = self.options.len().min(rng(self.options.len()));
        self.options.get(selection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn can_choose_a_single_element_from_a_list() {
        let rule = GeneratorRule::new(&["One", "Two"]);
        let selection = rule.Generate(|_| 1);
        assert_eq!(selection.unwrap().to_string(), "Two");
    }
}
