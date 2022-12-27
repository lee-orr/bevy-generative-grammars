#[cfg(feature = "asset")]
/// This module provides an asset loader for tracery grammars, allowing them to be used as assets as well
pub mod tracery_asset;

#[cfg(feature = "serde")]
pub use self::deserialize::*;
use crate::generator::*;
#[cfg(feature = "bevy")]
use bevy::{
    prelude::{Component, Resource},
    utils::HashMap,
};
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(not(feature = "bevy"))]
use std::collections::HashMap;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy", derive(Component, Resource))]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "asset", derive(bevy::reflect::TypeUuid))]
#[cfg_attr(feature = "asset", uuid = "40183015-2c4e-44d0-91ea-8028d45af39d")]
/// This is a grammar that handles rules provided in a tracery syntax.
/// See - <https://github.com/galaxykate/tracery> for more info on Tracery.
pub struct TraceryGrammar {
    rules: HashMap<String, Vec<String>>,
    keys: Vec<String>,
    starting_point: String,
}

#[cfg(feature = "serde")]
mod deserialize {
    use super::*;
    use serde::{Deserialize};

    #[derive(Deserialize)]
    struct TraceryGrammarContent {
        rules: HashMap<String, Vec<String>>,
        starting_point: Option<String>,
    }

    impl<'de> Deserialize<'de> for TraceryGrammar {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            match TraceryGrammarContent::deserialize(deserializer) {
                Ok(TraceryGrammarContent {
                    rules,
                    starting_point,
                }) => {
                    let keys = rules.keys().cloned().collect();
                    let starting_point = starting_point.unwrap_or("origin".to_string());
                    Ok(TraceryGrammar {
                        rules,
                        keys,
                        starting_point,
                    })
                }
                Err(err) => Err(err),
            }
        }
    }
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
        Self::generate_at(&grammar.default_starting_point().clone(), grammar, rng)
    }

    fn generate_at<R: FnMut(usize) -> usize>(
        key: &String,
        grammar: &TraceryGrammar,
        rng: &mut R,
    ) -> Option<String> {
        let initial = grammar.select_from_rule(key, rng);
        initial.map(|initial| Self::expand_from(initial, grammar, rng))
    }

    fn expand_from<R: FnMut(usize) -> usize>(
        initial: &String,
        grammar: &TraceryGrammar,
        rng: &mut R,
    ) -> String {
        let mut queue = vec![initial.clone()];
        let mut depth = 0;
        let mut result = initial.clone();
        while let Some(current) = queue.pop() {
            result = current;
            let ready = match grammar.apply_token_stream(&[&result], rng) {
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
                queue.push(result.clone());
            }
        }
        result
    }
}

/// This is a stateful string generator based on the tracery grammar. Note that since it is stateful, it does support variables.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy", derive(Component, Resource))]
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
                for (rule_key, value) in rules_to_apply.iter() {
                    match value {
                        MetaRuleProcessingResult::ProcessImmediately(stream) => {
                            queue.push((Some(rule_key.clone()), stream.clone()))
                        }
                        MetaRuleProcessingResult::ProcessWhenUsed(rule_value) => {
                            self.0
                                .set_additional_rules(rule_key.clone(), &[rule_value.to_string()]);
                        }
                    };
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
    ) -> (
        String,
        Vec<(String, MetaRuleProcessingResult<String, String>)>,
    ) {
        let mut new_rules = vec![];
        let mut inside = false;
        let result = result
            .split('[')
            .filter_map(|v| {
                if inside {
                    let mut split = v.split(']');
                    if let Some(inner) = split.next() {
                        let mut split = inner.split_inclusive(&[':', '|']);
                        if let (Some(key), Some(value)) = (split.next(), split.next()) {
                            if key.ends_with(':') {
                                new_rules.push((
                                    key[0..key.len() - 1].to_string(),
                                    MetaRuleProcessingResult::ProcessImmediately(value.to_string()),
                                ));
                            } else {
                                new_rules.push((
                                    key[0..key.len() - 1].to_string(),
                                    MetaRuleProcessingResult::ProcessWhenUsed(value.to_string()),
                                ));
                            }
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

    const RULES: &[(&str, &[&str])] = &[
    (
        "origin",
        &["[hero:#creature#][obstacle:#noun#]#intro# there was a #hero# that #encountered# #article##obstacle#."],
    ),
    (   "next",
        &[
            "Then, the #hero# decided to #action# #definite# #obstacle#.",
            "Our adventerous #hero# was ready to #action# #definite# #obstacle#."
        ]
    ),
    (   "finally",
        &[
            "And so, despite #finale# - our #hero# made it back home.",
            "And so - after #finale# - the lonely #hero# had proven their worth."
        ]
    ),
    (
        "intro",
        &[
            "once upon a time",
            "many years ago",
            "a long time ago",
            "in a far away land",
        ],
    ),
    ("creature", &["ant", "rabbit", "giraffe", "lion"]),
    ("encountered", &["ran into", "found", "saw"]),
    ("move", &["cross[finale|#went#]","circle[finale|#went#]","explore[finale|#went#]"]),
    ("fight", &["avoid[finale|#went#]", "fight[finale|#fought#]", "challange[finale|#fought#]"]),
    ("went", &["a long long journey", "a challanging path"]),
    ("fought", &["a challanging encounter", "a very close call"]),
    ("noun", &["[article:a][definite:the][action|#move#]river", "[article:a][definite:the][action|#move#]mountain", "[article:some][definite:them][action|#fight#]monsters"]),
];

    #[test]
    pub fn stateful_generator_works_with_complex_cases() {
        let mut generator = StatefulStringGenerator::new(RULES, None);
        let selection = generator.generate(&mut |_| 1);
        assert_eq!(
            selection.unwrap(),
            "many years ago there was a rabbit that found amountain."
        );
        let next = generator.generate_at(&"next".to_string(), &mut |_| 1);
        assert_eq!(
            next.unwrap(),
            "Our adventerous rabbit was ready to circle the mountain."
        );
        let finally = generator.generate_at(&"finally".to_string(), &mut |_| 1);
        assert_eq!(
            finally.unwrap(),
            "And so - after a challanging path - the lonely rabbit had proven their worth."
        );
    }
}
