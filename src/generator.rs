/// This defines a portion of a token stream that may be replaced using a rule, or might already be ready
pub enum Replacable<ResultType, RuleKeyType> {
    /// The value is already in it's final form
    Ready(ResultType),
    /// The value can be replaced by the provided rule
    Replace(RuleKeyType),
}

/// This trait defines an interface for a grammar
pub trait Grammar<RuleKeyType, ResultType: Clone> {
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
        stream: &[&ResultType],
    ) -> (bool, Vec<Replacable<ResultType, RuleKeyType>>);

    /// Selects an element from a rule's options. provides a default implementation in case no weighting is ncessessary.
    /// The RNG function should accept the total number of options, and return a single id (a number less than the total). If it is larger, we will use the last element.
    fn select_from_rule<R: FnMut(usize) -> usize>(
        &self,
        rule: &RuleKeyType,
        rng: &mut R,
    ) -> Option<&ResultType> {
        if let Some(options) = self.get_rule_options(rule) {
            let len = options.len() - 1;
            let index = len.min(rng(len));
            options.get(index)
        } else {
            None
        }
    }

    /// Takes a token stream, checks it for replacements, and then applies them by using select from rule.
    /// It returns a bool indicating whether it had to make any replacements this round, and a vec of the results.
    fn apply_token_stream<R: FnMut(usize) -> usize>(
        &self,
        stream: &[&ResultType],
        mut rng: &mut R,
    ) -> (bool, Vec<Option<ResultType>>) {
        let (is_done, values) = self.check_token_stream(stream);
        (
            is_done,
            values
                .iter()
                .map(|v| match v {
                    Replacable::Ready(v) => Some(v.clone()),
                    Replacable::Replace(v) => self.select_from_rule(v, &mut rng).cloned(),
                })
                .collect(),
        )
    }
}

/// A trait for Stateful grammars - grammars that can evolve with additional rules over time.
pub trait StatefulGrammar<RuleKeyType, ResultType: Clone>:
    Grammar<RuleKeyType, ResultType> + Clone
{
    /// This is a function for setting a new rule. The expectation is that it overrides the original.
    fn set_additional_rules(&mut self, rule: RuleKeyType, values: &[ResultType]);
}

/// This trait represents a stateless generator. You pass the grammar & rng in, and it can provide the resulting stream.
pub trait Generator<
    RuleKeyType: Clone,
    GrammarResultType: Clone,
    GrammarType: Grammar<RuleKeyType, GrammarResultType>,
    StreamType,
>
{
    /// This function generates a new value of `StreamType`, starting from the grammar's default rule
    fn generate<R: FnMut(usize) -> usize>(grammar: &GrammarType, rng: &mut R)
        -> Option<StreamType>;

    /// This function generates a new value of `StreamType`, starting from a provided rule key
    fn generate_at<R: FnMut(usize) -> usize>(
        key: &RuleKeyType,
        grammar: &GrammarType,
        rng: &mut R,
    ) -> Option<StreamType>;

    /// This function generates a new value of `StreamType`, starting by processing an initial input of `StreamType`
    fn expand_from<R: FnMut(usize) -> usize>(
        initial: &StreamType,
        grammar: &GrammarType,
        rng: &mut R,
    ) -> StreamType;
}

/// This enum helps handling comples meta-commands within a stream.
pub enum MetaRuleProcessingResult<GrammarResultType, StreamType> {
    /// This will immediately process the result of the command, so the processed result is consistent when the new rule is used
    ProcessImmediately(StreamType),
    /// This will store the result directly in the ruleset, and process it only when called - allowing for multiple possible results
    ProcessWhenUsed(GrammarResultType),
}

/// This trait represents a stateful generator. Here the generator owns the grammar, allowing it to make adjustments as needed.
pub trait StatefulGenerator<
    RuleKeyType: Clone,
    GrammarResultType: Clone,
    GrammarType: StatefulGrammar<RuleKeyType, GrammarResultType>,
    StreamType,
>
{
    /// This sets the used grammar
    fn set_grammar(&mut self, grammar: &GrammarType);
    /// This gets an immutable reference to the grammar
    fn get_grammar(&self) -> &GrammarType;
    /// This gets a mutable reference to the grammar
    fn get_grammar_mut(&mut self) -> &mut GrammarType;

    /// This function generates a new value of `StreamType`, starting from the grammar's default rule
    fn generate<R: FnMut(usize) -> usize>(&mut self, rng: &mut R) -> Option<StreamType>;

    /// This function generates a new value of `StreamType`, starting from a provided rule key
    fn generate_at<R: FnMut(usize) -> usize>(
        &mut self,
        key: &RuleKeyType,
        rng: &mut R,
    ) -> Option<StreamType>;

    /// This function generates a new value of `StreamType`, starting by processing an initial input of `StreamType`
    fn expand_from<R: FnMut(usize) -> usize>(
        &mut self,
        initial: &StreamType,
        rng: &mut R,
    ) -> StreamType;

    /// This function processes a stream, and determines which rules require updating.
    /// The result is a tuple, containing the following:
    /// - A version of the stream without the meta commands (that update the ruleset)
    /// - A vec of (`RuleKeyType`, `MetaRuleProcessingResult`)
    fn grab_rules_from_result<R: FnMut(usize) -> usize>(
        &mut self,
        result: &StreamType,
        rng: &mut R,
    ) -> (
        StreamType,
        Vec<(
            RuleKeyType,
            MetaRuleProcessingResult<RuleKeyType, StreamType>,
        )>,
    );
}
