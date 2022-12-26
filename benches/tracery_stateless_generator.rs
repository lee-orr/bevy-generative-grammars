use bevy_generative_grammars::{generator::*, tracery::*};
use criterion::{criterion_group, criterion_main, Criterion};

criterion_group!(benches, stateless_generator);
criterion_main!(benches);

const SIMPLE_GRAMMAR_DEFINITION : &[(&str, &[&str])] = &[
	("hero", &["Arjun"]),
	("heroPet", &["unicorn"]),
	("mood", &["vexed","indignant","impassioned","wistful","astute","courteous"]),
	("story", &["#hero# traveled with her pet #heroPet#.  #hero# was never #mood#, for the #heroPet# was always too #mood#."]),
	("origin", &["#story#"])
];

fn stateless_generator(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("stateless_genertor");
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(4));

    for num_runs in (1..5).map(|i| i * 2 * 1000) {
        group.bench_function(format!("{num_runs}_generated_stories"), |bencher| {
            bencher.iter(|| {
                let mut next_value = 0;
                let mut rng = |len| {
                    let value = next_value;
                    if next_value + 1 < len {
                        next_value += 1;
                    } else {
                        next_value = 0;
                    }

                    value
                };
                let grammar = TraceryGrammar::new(SIMPLE_GRAMMAR_DEFINITION, None);
                for _ in 0..num_runs {
                    let _ = StringGenerator::generate(&grammar, &mut rng);
                }
            });
        });
    }

    group.finish();
}
