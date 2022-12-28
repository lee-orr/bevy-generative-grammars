use bevy_generative_grammars::{
    generator::*,
    tracery::{StringGenerator, TraceryGrammar},
};
use rand::prelude::*;

const RULES: &[(&str, &[&str])] = &[
    (
        "origin",
        &["#intro# there was a #creature# that #acted# #noun#"],
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
    ("acted", &["ran into", "found", "saw"]),
    ("noun", &["a river", "a mountain", "some treasure"]),
];

fn main() {
    let grammar = TraceryGrammar::new(RULES, None);
    println!("Let me generate a story for you:");
    let mut rand = RandOwned::new(thread_rng());
    let story = StringGenerator::generate(&grammar, &mut rand);
    match story {
        Some(story) => {
            println!("{story}");
        }
        None => {
            eprint!("Couldn't generate story...");
        }
    }
}
