use bevy::prelude::*;
use bevy_generative_grammars::{
    generator::*,
    tracery::{tracery_asset::TraceryAssetPlugin, StatefulStringGenerator, TraceryGrammar},
};
use bevy_turborand::rng::Rng;

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
    App::new()
        .set_runner(terminal_runner)
        .add_plugin(TaskPoolPlugin::default())
        .add_plugin(TypeRegistrationPlugin::default())
        .add_plugin(AssetPlugin::default())
        .add_plugin(TraceryAssetPlugin::new().with_json(&["json"]))
        .add_startup_system(setup)
        .add_system(loaded_grammar)
        .add_system(progress_story)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let grammar = GrammarHandle(asset_server.load("story.json"), false);
    commands.insert_resource(grammar);
}

fn loaded_grammar(
    mut commands: Commands,
    mut handle: ResMut<GrammarHandle>,
    grammars: ResMut<Assets<TraceryGrammar>>,
) {
    if handle.1 {
        return;
    }

    if let Some(grammar) = grammars.get(&handle.0) {
        handle.1 = true;
        println!("Starting New Story!");
        commands.spawn((
            StatefulStringGenerator::clone_grammar(grammar),
            NextPrompt("origin".to_string()),
        ));
    } else {
        println!("Loading Grammar File...");
    }
}

#[derive(Component)]
struct NextPrompt(String);

#[derive(Resource)]
struct GrammarHandle(Handle<TraceryGrammar>, bool);

fn progress_story(
    mut commands: Commands,
    mut handle: ResMut<GrammarHandle>,
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
                    handle.1 = false;
                }
            }
        } else {
            eprintln!("failed to generate...");
        }
    }

    println!("Continue...")
}
