use crate::generator::*;
use bevy::{
    prelude::{Component, Resource},
    utils::HashMap,
};

#[derive(Debug, Clone, Resource, Component)]
/// This is a grammar that handles rules provided in a tracery syntax.
/// See - <https://github.com/galaxykate/tracery> for more info on Tracery.
pub struct TraceryGrammar {
    rules: HashMap<String, Vec<String>>,
    keys: Vec<String>,
    starting_point: String,
}

impl TraceryGrammar {
    /// This provides a new tracery grammar.
    /// You provide a set of rules as (Key, &[Values]) and optionally a starting point.
    /// If no starting point is provided, we fall back on "origin"
    pub fn new<T: Clone + Into<String>>(rules: &[(T, &[T])], starting_point: Option<T>) -> Self {
        Self {
            rules: rules
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone().into(),
                        v.iter().map(|v| v.clone().into()).collect(),
                    )
                })
                .collect(),
            keys: rules.iter().map(|(k, _)| k.clone().into()).collect(),
            starting_point: if let Some(starting_point) = starting_point {
                starting_point.into()
            } else {
                "origin".into()
            },
        }
    }
}

impl Grammar<String, String> for TraceryGrammar {
    fn rule_keys(&self) -> &Vec<String> {
        &self.keys
    }

    fn has_rule(&self, rule: &String) -> bool {
        self.rules.contains_key(rule)
    }

    fn default_starting_point(&self) -> &String {
        &self.starting_point
    }

    fn get_rule_options(&self, rule: &String) -> Option<&Vec<String>> {
        self.rules.get(rule)
    }

    fn check_token_stream(&self, stream: &[&String]) -> (bool, Vec<Replacable<String, String>>) {
        let mut ready = true;
        let mut has_replacements = false;
        let result = stream
            .iter()
            .map(|v| v.to_string())
            .fold("".to_string(), |s, v| format!("{s}{v}"))
            .split('#')
            .map(|v| {
                if ready {
                    ready = false;
                    Replacable::Ready(v.to_string())
                } else {
                    ready = true;
                    has_replacements = true;
                    Replacable::Replace(v.to_string())
                }
            })
            .collect();
        (has_replacements, result)
    }
}

impl StatefulGrammar<String, String> for TraceryGrammar {
    fn set_additional_rules(&mut self, rule: String, values: &[String]) {
        self.keys.push(rule.clone());
        self.rules.insert(rule, values.into());
    }
}

/// This is a stateless string generator based on the tracery grammar. Note that, since it's stateless, it does not support variables.
pub struct StringGenerator;

const MAX_DEPTH: usize = 50usize;

impl Generator<String, String, TraceryGrammar, String> for StringGenerator {
    fn generate<R: FnMut(usize) -> usize>(grammar: &TraceryGrammar, rng: &mut R) -> Option<String> {
        let initial = grammar.select_from_rule(grammar.default_starting_point(), rng);
        if let Some(initial) = initial {
            let mut result = initial.clone();
            let mut depth = 0;
            while let (true, results) = grammar.apply_token_stream(&[&result], rng) {
                result = results
                    .iter()
                    .filter_map(|v| v.as_ref())
                    .fold("".to_string(), |a, v| format!("{a}{v}"));
                depth += 1;
                if depth >= MAX_DEPTH {
                    break;
                }
            }
            Some(result)
        } else {
            None
        }
    }

    fn generate_at<R: FnMut(usize) -> usize>(
        _key: &String,
        _grammar: &TraceryGrammar,
        _rng: &mut R,
    ) -> Option<String> {
        todo!()
    }

    fn expand_from<R: FnMut(usize) -> usize>(
        _initial: &String,
        _grammar: &TraceryGrammar,
        _rng: &mut R,
    ) -> Option<String> {
        todo!()
    }
}

/// This is a stateful string generator based on the tracery grammar. Note that since it is stateful, it does support variables.
#[derive(Debug, Clone, Resource, Component)]
pub struct StatefulStringGenerator(TraceryGrammar);

impl StatefulStringGenerator {
    /// This provides a new stateful generating using tracery grammar.
    /// You provide a set of rules as (Key, &[Values]) and optionally a starting point.
    /// If no starting point is provided, we fall back on "origin"
    pub fn new<T: Clone + Into<String>>(rules: &[(T, &[T])], starting_point: Option<T>) -> Self {
        let grammar = TraceryGrammar::new(rules, starting_point);
        Self(grammar)
    }

    /// This creates a new stateful string generator by cloning an existing tracery grammar.
    pub fn clone_grammar(grammar: &TraceryGrammar) -> Self {
        Self(grammar.clone())
    }

    /// This creates a stateful generator wrapping an existing grammar.
    pub fn from_grammar(grammar: TraceryGrammar) -> Self {
        Self(grammar)
    }
}

impl StatefulGenerator<String, String, TraceryGrammar, String> for StatefulStringGenerator {
    fn generate<R: FnMut(usize) -> usize>(&mut self, rng: &mut R) -> Option<String> {
        let key = self.get_grammar().default_starting_point().clone();
        self.generate_at(&key, rng)
    }

    fn generate_at<R: FnMut(usize) -> usize>(
        &mut self,
        key: &String,
        rng: &mut R,
    ) -> Option<String> {
        let initial = self.get_grammar().select_from_rule(key, rng);
        initial
            .cloned()
            .map(|initial| self.expand_from(&initial, rng))
    }

    fn expand_from<R: FnMut(usize) -> usize>(
        &mut self,
        initial: &String,
        mut rng: &mut R,
    ) -> String {
        let mut queue = vec![(None, initial.clone())];
        let mut depth = 0;
        let mut result = initial.clone();
        while let Some((target, current)) = queue.pop() {
            let (initial, rules_to_apply) = self.grab_rules_from_result(&current, &mut rng);
            result = initial.clone();

            if !rules_to_apply.is_empty() {
                queue.push((target, result.clone()));
                for (rule, value) in rules_to_apply.iter() {
                    queue.push((Some(rule.clone()), value.clone()));
                }
                continue;
            }

            let ready = match self.get_grammar().apply_token_stream(&[&result], rng) {
                (true, results) => {
                    result = results
                        .iter()
                        .filter_map(|v| v.as_ref())
                        .fold("".to_string(), |a, v| format!("{a}{v}"));
                    false
                }
                _ => true,
            };

            depth += 1;
            if depth >= MAX_DEPTH {
                break;
            }
            if !ready {
                queue.push((target, result.clone()));
            } else if let Some(target) = target {
                self.0.set_additional_rules(target, &[result.clone()]);
            }
        }
        result
    }

    fn set_grammar(&mut self, grammar: &TraceryGrammar) {
        self.0 = grammar.clone()
    }

    fn get_grammar(&self) -> &TraceryGrammar {
        &self.0
    }

    fn get_grammar_mut(&mut self) -> &mut TraceryGrammar {
        &mut self.0
    }

    fn grab_rules_from_result<R: FnMut(usize) -> usize>(
        &mut self,
        result: &String,
        _rng: &mut R,
    ) -> (String, Vec<(String, String)>) {
        let mut inside = false;
        let mut new_rules = vec![];
        let result = result
            .split('[')
            .filter_map(|v| {
                if inside {
                    inside = false;
                    let mut split = v.split(']');
                    if let Some(inner) = split.next() {
                        let mut split = inner.split(':');
                        if let (Some(key), Some(value)) = (split.next(), split.next()) {
                            new_rules.push((key.to_string(), value.to_string()));
                        }
                    } else {
                        return None;
                    }
                    Some(split.fold("".to_string(), |a, b| format!("{a}{b}")))
                } else {
                    inside = true;
                    Some(v.to_string())
                }
            })
            .fold("".to_string(), |a, b| format!("{a}{b}"));
        (result, new_rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn can_choose_a_single_element_from_a_list() {
        let rule = TraceryGrammar::new(&[("default", &["One", "Two"])], Some("default"));
        let mut select = 0;
        let mut fun = |_| {
            let result = select;
            select += 1;
            result
        };

        assert_eq!(StringGenerator::generate(&rule, &mut fun).unwrap(), "One");
        assert_eq!(StringGenerator::generate(&rule, &mut fun).unwrap(), "Two");
        assert_eq!(StringGenerator::generate(&rule, &mut fun).unwrap(), "Two");
    }

    #[test]
    pub fn element_replacer_can_replace_elements_with_other_valid_elements() {
        let rule = TraceryGrammar::new(
            &[("default", &["One", "#Two#"]), ("Two", &["Three", "Four"])],
            Some("default"),
        );
        let selection = StringGenerator::generate(&rule, &mut |_| 1);
        assert_eq!(selection.unwrap(), "Four");
    }

    #[test]
    pub fn element_replacer_can_replace_elements_with_other_valid_elements_at_depth() {
        let rule = TraceryGrammar::new(
            &[
                ("default", &["One", "#Two#"]),
                ("Two", &["Three", "#Four#"]),
                ("Four", &["What"]),
            ],
            Some("default"),
        );
        let selection = StringGenerator::generate(&rule, &mut |_| 1);
        assert_eq!(selection.unwrap(), "What");
    }

    #[test]
    pub fn stateful_can_choose_a_single_element_from_a_list() {
        let rule = TraceryGrammar::new(&[("default", &["One", "Two"])], Some("default"));
        let mut select = 0;
        let mut fun = |_| {
            let result = select;
            select += 1;
            result
        };

        let mut generator = StatefulStringGenerator(rule);

        assert_eq!(generator.generate(&mut fun).unwrap(), "One");
        assert_eq!(generator.generate(&mut fun).unwrap(), "Two");
        assert_eq!(generator.generate(&mut fun).unwrap(), "Two");
    }

    #[test]
    pub fn stateful_element_replacer_can_replace_elements_with_other_valid_elements() {
        let rule = TraceryGrammar::new(
            &[("default", &["One", "#Two#"]), ("Two", &["Three", "Four"])],
            Some("default"),
        );
        let selection = StatefulStringGenerator(rule).generate(&mut |_| 1);
        assert_eq!(selection.unwrap(), "Four");
    }

    #[test]
    pub fn stateful_element_replacer_can_replace_elements_with_other_valid_elements_at_depth() {
        let rule = TraceryGrammar::new(
            &[
                ("default", &["One", "#Two#"]),
                ("Two", &["Three", "#Four#"]),
                ("Four", &["What"]),
            ],
            Some("default"),
        );
        let selection = StatefulStringGenerator(rule).generate(&mut |_| 1);
        assert_eq!(selection.unwrap(), "What");
    }

    #[test]
    pub fn stateful_generator_can_set_value_and_use_it_later() {
        let rule = TraceryGrammar::new(
            &[
                ("default", &["One", "[val:#Two#]Hi #val#"]),
                ("Two", &["Three", "#Four#"]),
                ("Four", &["What is going on?"]),
            ],
            Some("default"),
        );
        let selection = StatefulStringGenerator(rule).generate(&mut |_| 1);
        assert_eq!(selection.unwrap(), "Hi What is going on?");
    }
}
