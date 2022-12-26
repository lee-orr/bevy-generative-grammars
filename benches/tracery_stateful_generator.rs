use bevy_generative_grammars::{generator::*, tracery::*};
use criterion::{criterion_group, criterion_main, Criterion};

criterion_group!(benches, stateful_generator_complex, stateful_generator);
criterion_main!(benches);

const COMPLEX_GRAMMAR_DEFINITION : &[(&str, &[&str])] = &[
	("name", &["Arjun","Yuuma","Darcy","Mia","Chiaki","Izzi","Azra","Lina"]),
	("animal", &["unicorn","raven","sparrow","scorpion","coyote","eagle","owl","lizard","zebra","duck","kitten"]),
	("mood", &["vexed","indignant","impassioned","wistful","astute","courteous"]),
	("story", &["#hero# traveled with her pet #heroPet#.  #hero# was never #mood#, for the #heroPet# was always too #mood#."]),
	("origin", &["#[hero:#name#][heroPet:#animal#]story#"])
];

const SIMPLE_GRAMMAR_DEFINITION : &[(&str, &[&str])] = &[
	("hero", &["Arjun"]),
	("heroPet", &["unicorn"]),
	("mood", &["vexed","indignant","impassioned","wistful","astute","courteous"]),
	("story", &["#hero# traveled with her pet #heroPet#.  #hero# was never #mood#, for the #heroPet# was always too #mood#."]),
	("origin", &["#story#"])
];

fn stateful_generator_complex(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("stateful_generator_complex");
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(4));

    for num_runs in (1..5).map(|i| i * 2 * 1000) {
        group.bench_function(format!("{num_runs}_generated_complex_stories"), |bencher| {
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
                let mut generator = StatefulStringGenerator::new(COMPLEX_GRAMMAR_DEFINITION, None);
                for _ in 0..num_runs {
                    let _ = generator.generate(&mut rng);
                }
            });
        });
    }

    group.finish();
}

fn stateful_generator(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("stateful_generator_simple");
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
                let mut generator = StatefulStringGenerator::new(SIMPLE_GRAMMAR_DEFINITION, None);
                for _ in 0..num_runs {
                    let _ = generator.generate(&mut rng);
                }
            });
        });
    }

    group.finish();
}
