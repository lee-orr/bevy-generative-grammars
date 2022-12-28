use std::{fmt::Debug, collections::VecDeque};

use bevy::prelude::Res;

#[derive(Clone, PartialEq, Debug)]
/// This defines a portion of a token stream that may be replaced using a rule, or might already be ready
pub enum Replacable<RuleKeyType: Clone + PartialEq + Debug, ResultType: Clone + PartialEq + Debug> {
    /// The value is already in it's final form
    Ready(ResultType),
    /// The value can be replaced by the provided rule
    Replace(RuleKeyType),
    /// The value is a meta rule for immediate processing
    ImmediateMeta(RuleKeyType, ResultType),
    /// The value is a meta rule for delayed processing - basically aliasing the rule
    DelayedMeta(RuleKeyType, ResultType)
}

/// This sets the direction of processing for the grammar
#[derive(Clone, Copy, Debug)]
pub enum GrammarProcessingDirection {
    /// Breadth first means it first iterates once through an entire stream - only making initial replacements, but not processing their values yet.
    /// It then is able to apply the next iteration - until it either reaches a maximum depth or stabilizes.
    BreadthFirst,
    /// Depth first means it goes in sequence through the stream, and each time it is able to make a replacement it replaces it as far as it can go.
    /// This will not evolve, and cannot detect new emergent patterns in a stream, but can be very effective for specific contexts like text generation.
    DepthFirst
}

impl Default for GrammarProcessingDirection {
    fn default() -> Self {
        Self::BreadthFirst
    }
}

/// This trait defines a random number generator capable of choosing a single item from a list of len usize.
/// It is used for selecting a rule for using when multiple rules are available.
pub trait GrammarRandomNumberGenerator {
    /// This function provides a random number between 0 and len
    fn get_number(&mut self, len: usize) -> usize;
}

impl<T:FnMut(usize) -> usize>  GrammarRandomNumberGenerator for T {
    fn get_number(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        self(len)
    }
}

impl GrammarRandomNumberGenerator for usize {
    fn get_number(&mut self, _: usize) -> usize {
        self.clone()
    }
}

/// This trait defines an interface for a grammar
pub trait Grammar<RuleKeyType: Clone + PartialEq + Debug, ResultType: Clone + PartialEq + Debug, StreamType: Clone + PartialEq + Debug> {
    /// Gets a Vec of all the possible rule keys - can be used to see if any match
    fn rule_keys(&self) -> &Vec<RuleKeyType>;
    /// Checks if a given rule key is available
    fn has_rule(&self, rule: &RuleKeyType) -> bool;
    /// Gets all the possible expansions from a rule key
    fn get_rule_options(&self, rule: &RuleKeyType) -> Option<&Vec<ResultType>>;
    /// Gets the default starting key - used if no other key is set
    fn default_starting_point(&self) -> &RuleKeyType;

    /// Parses a token stream and determines a) whether there are any tokens to replace and b) if so, which
    /// The bool is true if there are no more tokens that need replacing
    fn check_token_stream(
        &self,
        stream: &StreamType,
    ) -> (bool, Vec<Replacable<RuleKeyType, ResultType>>);

    /// Selects an element from a rule's options. provides a default implementation in case no weighting is ncessessary.
    /// The RNG function should accept the total number of options, and return a single id (a number less than the total). If it is larger, we will use the last element.
    fn select_from_rule<R: GrammarRandomNumberGenerator>(
        &self,
        rule: &RuleKeyType,
        rng: &mut R,
    ) -> Option<&ResultType> {
        if let Some(options) = self.get_rule_options(rule) {
            let len = options.len();
            let max = len.checked_sub(1).unwrap_or_default();
            let rng = rng.get_number(len);
            let index = max.min(rng);
            println!("Selecting from - len: {}, max id: {}, rng: {}, selected: {}",len, max, rng, index);
            options.get(index)
        } else {
            None
        }
    }

    /// Converts a rule key to a default result, in case no matching rule is found in the grammar.
    fn rule_to_default_result(&self, rule: &RuleKeyType) -> ResultType;

    /// Converts a group result types to a stream type
    fn result_to_stream(&self, result: &[ResultType]) -> StreamType;

    /// Converts a stream to a vec of result type
    fn stream_to_result(&self, stream: &StreamType) -> Vec<ResultType>;

    /// determines if the grammar should be processed breadth-first or depth-first
    fn processing_direction(&self) -> GrammarProcessingDirection;

    
    /// This is a function for setting a new rule. The expectation is that it overrides the original.
    fn set_additional_rules(&mut self, rule: RuleKeyType, values: &[ResultType]);

    /// This is used to clone all the roles from another grammar into this one. This is used by stateful generators to update their state.
    fn copy_and_replace_rules(&mut self, other: &Self) {
        for rule in other.rule_keys() {
            if let Some(values) = other.get_rule_options(rule) {
                let rule = rule.clone();
                self.set_additional_rules(rule, values);
            }
        }
    }

    /// Provides the maximum depth (number of iterations) allowed for the generator. It will always quit early if it stabilizes,
    /// but otherwise it will conclude when it reaches the provided depth.
    fn max_depth(&self) -> usize {
        50
    }

    /// Takes a token stream, checks it for replacements, and then applies them by using select from rule.
    /// It returns a bool indicating whether it had to make any replacements this round, and a vec of the results.
    fn process_stream<R: GrammarRandomNumberGenerator>(
        &self,
        stream: &StreamType,
        rng: &mut R,
        temporary_grammar: &mut Self
    ) -> StreamType {
        println!("Applying Token Stream to {:?}", stream);

        match self.processing_direction() {
            GrammarProcessingDirection::BreadthFirst => {
                self.breadth_first_processing(stream, temporary_grammar, rng)
            },
            GrammarProcessingDirection::DepthFirst => todo!(),
        }
    }

    /// Processes a stream breadth first, regardless of the settings of the grammar 
    fn breadth_first_processing<R: GrammarRandomNumberGenerator>(&self, stream: &StreamType, temporary_grammar: &mut Self, rng: &mut R) -> StreamType {
        println!("Breadth First");
        let MAX_DEPTH = self.max_depth();
        let (skippable, initial) = self.check_token_stream(stream);
        if skippable {
            println!("Fully skippable");
            return stream.clone();
        }

        let mut queue: Vec<(Option<RuleKeyType>, Vec<Replacable<RuleKeyType, ResultType>>)> = vec![(None, initial)];
        let mut append_to_queue = vec![];
        let mut depth = 0;
        let mut result = stream.clone();
        let mut tmp_result = None;
        while let Some((target, current)) = queue.pop() {
            println!("**** ITERATION *****");
            println!("Target: {:?}\nInput:\n{:?}", &target, &current);
            let next = current.into_iter().filter_map(|token| {
               let result = match token {
                    Replacable::Ready(v) => Some(v),
                    Replacable::Replace(key) => {
                        if let Some(result) = temporary_grammar.select_from_rule(&key, rng) {
                            Some(result.clone())
                        } else if let Some(result) = self.select_from_rule(&key, rng) {
                            Some(result.clone())
                        } else {
                            Some(self.rule_to_default_result(&key))
                        }
                    },
                    Replacable::ImmediateMeta(key, value) => {
                        let stream = self.result_to_stream(&[value.clone()]);
                        let (skippable, replaceables) = self.check_token_stream(&stream);
                        if skippable {
                            temporary_grammar.set_additional_rules(key, &[value]);
                        } else {
                            append_to_queue.push((Some(key), replaceables));
                        }
                        None
                    },
                    Replacable::DelayedMeta(key, value) => {
                        temporary_grammar.set_additional_rules(key, &[value]);
                        None
                    },
                };
                result
            }).collect::<Vec<_>>();
    
            println!("Result:\n{:?}", &next);
    
            let next = self.result_to_stream(&next);
    
            println!("Stream:\n{:?}", &next);
    
            if let Some(target) = &target {
                println!("got a target");
                if let Some(tmp) = &tmp_result {
                
                    println!("and a temp result");
                    if tmp == &next {
                    
                println!("they match");
                        temporary_grammar.set_additional_rules(target.clone(), &self.stream_to_result(&next));
                        tmp_result = None;
                        continue;
                    }
                    println!("they don't match - carry on");
                }
                tmp_result = Some(next.clone());
            } else if result == next {
                println!("no target - and result = next - ending early");
                break;
            } else {
                println!("no target, keep going...");
                result = next.clone();
            }
    
            
            depth += 1;
            if depth >= MAX_DEPTH {
                break;
            }
        
            println!("Checking next stream:\n{:?}", next);
            let (skippable, next) = self.check_token_stream(&next);
            println!("Setting up for next stream:\nis skippable: {skippable}\n{:?}", next);
            if skippable {
                if let (Some(target), Some(tmp)) = (&target, tmp_result) {
                    temporary_grammar.set_additional_rules(target.clone(), &self.stream_to_result(&tmp));
                    tmp_result = None;
                    continue;
                } else {
                    break;
                }
            }
            queue.push((target, next));
            queue.append(&mut append_to_queue);
        }
        result
    }
    
}


/// This trait represents a stateless generator. You pass the grammar & rng in, and it can provide the resulting stream.
pub trait Generator<
    RuleKeyType: Clone + PartialEq + Debug,
    GrammarResultType: Clone + PartialEq + Debug,
    StreamType: Clone+ PartialEq + Debug,
    GrammarType: Grammar<RuleKeyType, GrammarResultType, StreamType>,
>
{
    /// This function generates a new value of `StreamType`, starting from the grammar's default rule
    fn generate<R: GrammarRandomNumberGenerator>(grammar: &GrammarType, rng: &mut R)
        -> Option<StreamType>;

    /// This function generates a new value of `StreamType`, starting from a provided rule key
    fn generate_at<R: GrammarRandomNumberGenerator>(
        key: &RuleKeyType,
        grammar: &GrammarType,
        rng: &mut R,
    ) -> Option<StreamType>;

    /// This function generates a new value of `StreamType`, starting by processing an initial input of `StreamType`
    fn expand_from<R: GrammarRandomNumberGenerator>(
        initial: &StreamType,
        grammar: &GrammarType,
        rng: &mut R,
    ) -> StreamType;
}

/// This enum helps handling comples meta-commands within a stream.
pub enum MetaRuleProcessingResult<RuleKey, GrammarResultType, StreamType> {
    /// This is just content
    Raw(StreamType),
    /// This will immediately process the result of the command, so the processed result is consistent when the new rule is used
    ImmediateMeta(RuleKey, StreamType),
    /// This will store the result directly in the ruleset, and process it only when called - allowing for multiple possible results
    DelayedMeta(RuleKey, GrammarResultType),
}

/// This trait represents a stateful generator. Here the generator owns the grammar, allowing it to make adjustments as needed.
pub trait StatefulGenerator<
    RuleKeyType: Clone + PartialEq + Debug,
    GrammarResultType: Clone + PartialEq + Debug,
    StreamType: Clone + PartialEq + Debug,
    GrammarType: Grammar<RuleKeyType, GrammarResultType, StreamType>,
>
{
    /// This sets the used grammar
    fn set_grammar(&mut self, grammar: &GrammarType);
    /// This gets an immutable reference to the grammar
    fn get_grammar(&self) -> &GrammarType;
    /// This gets a mutable reference to the grammar
    fn get_grammar_mut(&mut self) -> &mut GrammarType;

    /// This function generates a new value of `StreamType`, starting from the grammar's default rule
    fn generate<R: GrammarRandomNumberGenerator>(&mut self, rng: &mut R) -> Option<StreamType>;

    /// This function generates a new value of `StreamType`, starting from a provided rule key
    fn generate_at<R: GrammarRandomNumberGenerator>(
        &mut self,
        key: &RuleKeyType,
        rng: &mut R,
    ) -> Option<StreamType>;

    /// This function generates a new value of `StreamType`, starting by processing an initial input of `StreamType`
    fn expand_from<R: GrammarRandomNumberGenerator>(
        &mut self,
        initial: &StreamType,
        rng: &mut R,
    ) -> StreamType;
}
