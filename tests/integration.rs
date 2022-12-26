#![cfg(test)]

#[test]
fn always_true() {
    use bevy_generative_grammars::utils::returns_true;

    assert!(returns_true());
}
