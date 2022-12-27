use std::io::stdin;

use bevy_generative_grammars::{
    generator::*,
    tracery::{StatefulStringGenerator, TraceryGrammar},
};
use rand::prelude::*;

const RULES: &[(&str, &[&str])] =  &[
    (
        "origin",
        &["[hero:#creature#][obstacle:#noun#]#intro# there was a #hero# that #encountered# #article# #obstacle#."],
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

fn main() {
    let grammar = TraceryGrammar::new(RULES, None);
    println!("Let me generate a story for you:");
    let mut rng = thread_rng();
    let mut rng_func = |len| {
        if len == 0 {
            0
        } else {
            rng.gen_range(0..len)
        }
    };
    let mut generator = StatefulStringGenerator::from_grammar(grammar);
    let story = generator.generate(&mut rng_func);
    match story {
        Some(story) => {
            println!("{story}");
        }
        None => {
            eprint!("Couldn't generate story...");
            return;
        }
    }
    println!("Do you want to continue? y/n");
    let mut s = String::new();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if s.to_lowercase().contains('y') {
        let (line_1, line_2) = (
            generator.generate_at(&"next".to_string(), &mut rng_func),
            generator.generate_at(&"finally".to_string(), &mut rng_func),
        );
        match (line_1, line_2) {
            (Some(line_1), Some(line_2)) => {
                println!("{line_1}");
                println!("{line_2}");
            }
            _ => {
                eprint!("Couldn't generate story...");
            }
        }
    } else {
        println!("Ok - goodbye");
    }
}
