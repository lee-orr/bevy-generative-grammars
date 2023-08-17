use bevy::prelude::*;
use bevy_generative_grammars::{
    generator::*,
    tracery::{StatefulStringGenerator, TraceryGrammar},
};
use bevy_turborand::rng::Rng;

const RULES: &[(&str, &[&str])] =  &[
    (
        "origin",
        &["[hero:#creature#][obstacle:#noun#]#intro# there was a #hero# that #encountered# #article# #obstacle#.|next"],
    ),
    (   "next",
        &[
            "Then, the #hero# decided to #action# #definite# #obstacle#.|finally",
            "Our adventerous #hero# was ready to #action# #definite# #obstacle#.|finally"
        ]
    ),
    (   "finally",
        &[
            "And so, despite #finale# - our #hero# made it back home.|done",
            "And so - after #finale# - the lonely #hero# had proven their worth.|done"
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

fn terminal_runner(mut app: App) {
    println!("Press Enter to Progress, or type 'exit' to exit");
    for line in std::io::stdin().lines() {
        let typed: String = line.unwrap_or_default();
        if typed == "exit" {
            return;
        }
        app.update();
    }
}

fn main() {
    let grammar = TraceryGrammar::new(RULES, None);

    App::new()
        .set_runner(terminal_runner)
        .insert_resource(grammar)
        .add_systems(Startup, setup)
        .add_systems(Update, progress_story)
        .run();
}

fn setup(mut commands: Commands, grammar: Res<TraceryGrammar>) {
    commands.spawn((
        StatefulStringGenerator::clone_grammar(&grammar),
        NextPrompt("origin".to_string()),
    ));
}

#[derive(Component)]
struct NextPrompt(String);

fn progress_story(
    mut commands: Commands,
    mut query: Query<(Entity, &mut StatefulStringGenerator, &mut NextPrompt)>,
) {
    let mut rng = TurboRandOwned::new(Rng::new());

    for (entity, mut generator, mut next_prompt) in query.iter_mut() {
        if let Some(generated) = generator.generate_at(&next_prompt.0, &mut rng) {
            let mut split = generated.split('|');
            if let Some(generated) = split.next() {
                println!("{generated}");
            }
            if let Some(next_item) = split.next() {
                if next_item != "done" {
                    next_prompt.0 = next_item.to_string();
                } else {
                    commands.entity(entity).despawn();

                    println!("Story Complete...");
                }
            }
        } else {
            eprintln!("failed to generate...");
        }
    }

    println!("Continue...")
}
