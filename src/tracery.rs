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
#[cfg_attr(
    feature = "asset",
    derive(bevy::reflect::TypeUuid, bevy::reflect::TypePath, bevy::asset::Asset,)
)]
#[cfg_attr(feature = "asset", uuid = "40183015-2c4e-44d0-91ea-8028d45af39d")]
/// This is a grammar that handles rules provided in a tracery syntax.
/// See - <https://github.com/galaxykate/tracery> for more info on Tracery.
pub struct TraceryGrammar {
    rules: HashMap<String, Vec<String>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    keys: Vec<String>,
    starting_point: String,
}

#[cfg(feature = "serde")]
mod deserialize {
    use super::*;
    use serde::Deserialize;

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
    /// This provides an empty tracery grammar.
    /// Mostly used for handling stateless generators.
    pub fn empty() -> Self {
        Self {
            rules: Default::default(),
            keys: vec![],
            starting_point: "origin".to_string(),
        }
    }
    /// This provides a new tracery grammar.
    /// You provide a set of rules as `(Key, &[Values])` and optionally a starting point.
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

impl Grammar<String, String, String> for TraceryGrammar {
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

    fn check_token_stream(&self, stream: &String) -> (bool, Vec<Replacable<String, String>>) {
        let mut has_replacements = false;
        let mut has_meta = false;
        let mut inside = false;
        let result = stream
            .split('[')
            .flat_map(|v| {
                if inside {
                    has_meta = true;
                    let mut result = vec![];
                    let mut split = v.split(']');
                    if let Some(inner) = split.next() {
                        let mut split = inner.split_inclusive(&[':', '|']);
                        if let (Some(key), Some(value)) = (split.next(), split.next()) {
                            if key.ends_with(':') {
                                result.push(MetaRuleProcessingResult::ImmediateMeta(
                                    &key[0..key.len() - 1],
                                    value,
                                ));
                            } else {
                                result.push(MetaRuleProcessingResult::DelayedMeta(
                                    &key[0..key.len() - 1],
                                    value,
                                ));
                            }
                        } else {
                            result.push(MetaRuleProcessingResult::Raw(inner));
                        }
                    } else {
                        result.push(MetaRuleProcessingResult::Raw(v));
                    }
                    for v in split {
                        result.push(MetaRuleProcessingResult::Raw(v))
                    }
                    result
                } else {
                    inside = true;
                    vec![MetaRuleProcessingResult::Raw(v)]
                }
            })
            .flat_map(|v| match v {
                MetaRuleProcessingResult::Raw(v) => {
                    let mut ready = true;
                    v.split('#')
                        .filter_map(|v| {
                            if ready {
                                ready = false;
                                if v.is_empty() {
                                    return None;
                                }
                                Some(Replacable::Ready(v.to_string()))
                            } else {
                                ready = true;
                                has_replacements = true;
                                Some(Replacable::Replace(v.to_string()))
                            }
                        })
                        .collect::<Vec<_>>()
                }
                MetaRuleProcessingResult::ImmediateMeta(key, val) => {
                    vec![Replacable::ImmediateMeta(key.to_string(), val.to_string())]
                }
                MetaRuleProcessingResult::DelayedMeta(key, val) => {
                    vec![Replacable::DelayedMeta(key.to_string(), val.to_string())]
                }
            })
            .collect::<Vec<_>>();

        (!has_replacements && !has_meta, result)
    }

    fn rule_to_default_result(&self, rule: &String) -> String {
        format!("#{rule}#")
    }

    fn processing_direction(&self) -> GrammarProcessingDirection {
        GrammarProcessingDirection::DepthFirst
    }

    fn result_to_stream(&self, result: &[String]) -> String {
        result.join("")
    }

    fn set_additional_rules(&mut self, rule: String, values: &[String]) {
        self.keys.push(rule.clone());
        self.rules.insert(rule, values.into());
    }

    fn stream_to_result(&self, stream: &String) -> Vec<String> {
        vec![stream.clone()]
    }
}

/// This is a stateless string generator based on the tracery grammar. Note that, since it's stateless, it does not support variables.
pub struct StringGenerator;

impl Generator<String, String, String, TraceryGrammar> for StringGenerator {
    fn generate<R: GrammarRandomNumberGenerator>(
        grammar: &TraceryGrammar,
        rng: &mut R,
    ) -> Option<String> {
        Self::generate_at(&grammar.default_starting_point().clone(), grammar, rng)
    }

    fn generate_at<R: GrammarRandomNumberGenerator>(
        key: &String,
        grammar: &TraceryGrammar,
        rng: &mut R,
    ) -> Option<String> {
        let initial = grammar.select_from_rule(key, rng);
        initial.map(|initial| Self::expand_from(initial, grammar, rng))
    }

    fn expand_from<R: GrammarRandomNumberGenerator>(
        initial: &String,
        grammar: &TraceryGrammar,
        rng: &mut R,
    ) -> String {
        let mut tmp = TraceryGrammar::empty();
        grammar.process_stream(initial, rng, &mut tmp)
    }
}

/// This is a stateful string generator based on the tracery grammar. Note that since it is stateful, it does support variables.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy", derive(Component, Resource))]
pub struct StatefulStringGenerator(TraceryGrammar);

impl StatefulStringGenerator {
    /// This provides a new stateful generating using tracery grammar.
    /// You provide a set of rules as `(Key, &[Values])` and optionally a starting point.
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

impl StatefulGenerator<String, String, String, TraceryGrammar> for StatefulStringGenerator {
    fn generate<R: GrammarRandomNumberGenerator>(&mut self, rng: &mut R) -> Option<String> {
        let key = self.get_grammar().default_starting_point().clone();
        self.generate_at(&key, rng)
    }

    fn generate_at<R: GrammarRandomNumberGenerator>(
        &mut self,
        key: &String,
        rng: &mut R,
    ) -> Option<String> {
        let initial = self.get_grammar().select_from_rule(key, rng);
        initial
            .cloned()
            .map(|initial| self.expand_from(&initial, rng))
    }

    fn expand_from<R: GrammarRandomNumberGenerator>(
        &mut self,
        initial: &String,
        rng: &mut R,
    ) -> String {
        let mut tmp = TraceryGrammar::empty();
        let result = self.get_grammar().process_stream(initial, rng, &mut tmp);
        self.get_grammar_mut().copy_and_replace_rules(&tmp);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn can_choose_a_single_element_from_a_list() {
        let rule = TraceryGrammar::new(&[("default", &["One", "Two"])], Some("default"));

        assert_eq!(StringGenerator::generate(&rule, &mut 0).unwrap(), "One");
        assert_eq!(StringGenerator::generate(&rule, &mut 1).unwrap(), "Two");
        assert_eq!(StringGenerator::generate(&rule, &mut 2).unwrap(), "Two");
    }

    #[test]
    pub fn element_replacer_can_replace_elements_with_other_valid_elements() {
        let rule = TraceryGrammar::new(
            &[("default", &["One", "#Two#"]), ("Two", &["Three", "Four"])],
            Some("default"),
        );
        let selection = StringGenerator::generate(&rule, &mut 1);
        assert_eq!(selection.unwrap(), "Four");
    }

    #[test]
    pub fn element_replacer_can_replace_elements_with_other_valid_elements_at_depth() {
        let rule = TraceryGrammar::new(
            &[
                ("default", &["One", "#Two#"]),
                ("Two", &["Three", "#Four# is up"]),
                ("Four", &["What"]),
            ],
            Some("default"),
        );
        let selection = StringGenerator::generate(&rule, &mut 1);
        assert_eq!(selection.unwrap(), "What is up");
    }

    #[test]
    pub fn stateful_can_choose_a_single_element_from_a_list() {
        let rule = TraceryGrammar::new(&[("default", &["One", "Two"])], Some("default"));
        let mut generator = StatefulStringGenerator(rule);

        assert_eq!(generator.generate(&mut 0).unwrap(), "One");
        assert_eq!(generator.generate(&mut 1).unwrap(), "Two");
        assert_eq!(generator.generate(&mut 2).unwrap(), "Two");
    }

    #[test]
    pub fn stateful_element_replacer_can_replace_elements_with_other_valid_elements() {
        let rule = TraceryGrammar::new(
            &[("default", &["One", "#Two#"]), ("Two", &["Three", "Four"])],
            Some("default"),
        );
        let selection = StatefulStringGenerator(rule).generate(&mut 1);
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
        let selection = StatefulStringGenerator(rule).generate(&mut 1);
        assert_eq!(selection.unwrap(), "What");
    }

    #[test]
    pub fn stateful_generator_can_set_value_and_use_it_later() {
        let rule = TraceryGrammar::new(
            &[
                ("default", &["One", "[val:#Two#]Hi #val#"]),
                ("Two", &["Three", "#Four#"]),
                ("Four", &["What is going on?"]),
                ("Five", &["Hey there"]),
            ],
            Some("default"),
        );
        let mut stateful_string_generator = StatefulStringGenerator(rule);
        let selection = stateful_string_generator.generate(&mut 1);
        assert_eq!(selection.unwrap(), "Hi What is going on?");
    }

    #[test]
    pub fn stateful_generator_can_set_value_at_depth_and_use_it_later() {
        let rule = TraceryGrammar::new(
            &[
                ("default", &["One", "#set#Hi #val#"]),
                ("set", &["[val:#Two#]"]),
                ("Two", &["Three", "#Four#"]),
                ("Four", &["What is going on?"]),
            ],
            Some("default"),
        );
        let mut stateful_string_generator = StatefulStringGenerator(rule);
        let selection = stateful_string_generator.generate(&mut 1);
        assert_eq!(selection.unwrap(), "Hi What is going on?");
    }

    #[test]
    pub fn stateful_generator_can_reset_a_value_at_depth_and_use_the_new_value() {
        let rule = TraceryGrammar::new(
            &[
                ("default", &["One", "#set#Hi #val#"]),
                ("set", &["[val:#Two#]"]),
                ("set_2", &["[val:#Five#]"]),
                ("Two", &["Three", "#Four# here"]),
                ("Four", &["What is going on"]),
                ("Five", &["Hey there"]),
            ],
            Some("default"),
        );
        let mut stateful_string_generator = StatefulStringGenerator(rule);
        let selection = stateful_string_generator.generate(&mut 1);
        assert_eq!(selection.unwrap(), "Hi What is going on here");
        let selection =
            stateful_string_generator.expand_from(&"#set_2#Oh #val#".to_string(), &mut 1);
        assert_eq!(selection, "Oh Hey there");
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
        let selection = generator.generate(&mut 1);
        assert_eq!(
            selection.unwrap(),
            "many years ago there was a rabbit that found amountain."
        );
        let next = generator.generate_at(&"next".to_string(), &mut 1);
        assert_eq!(
            next.unwrap(),
            "Our adventerous rabbit was ready to circle the mountain."
        );
        let finally = generator.generate_at(&"finally".to_string(), &mut 1);
        assert_eq!(
            finally.unwrap(),
            "And so - after a challanging path - the lonely rabbit had proven their worth."
        );
    }
}
