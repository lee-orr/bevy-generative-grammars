use super::GrammarRandomNumberGenerator;

/// This is a wrapper struct that allows use of any type that implements the `rand::Rng` trait
pub struct TurboRand<'a, T: bevy_turborand::TurboRand>(&'a mut T);

impl<'a, T: bevy_turborand::TurboRand> TurboRand<'a, T> {
    /// Wraps a type implementing `rand::Rng` for use with grammars
    pub fn new(rng: &'a mut T) -> Self {
        Self(rng)
    }
}

impl<'a, T: bevy_turborand::TurboRand> GrammarRandomNumberGenerator for TurboRand<'a, T> {
    fn get_number(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        self.0.usize(0..len)
    }
}

/// This is a wrapper struct that allows use of any type that implements the `rand::Rng` trait
/// Unlike Rand - this takes over the original object.
pub struct TurboRandOwned<T: bevy_turborand::TurboRand>(T);

impl<T: bevy_turborand::TurboRand> TurboRandOwned<T> {
    /// Wraps a type implementing `rand::Rng` for use with grammars
    pub fn new(rng: T) -> Self {
        Self(rng)
    }
}

impl<T: bevy_turborand::TurboRand> GrammarRandomNumberGenerator for TurboRandOwned<T> {
    fn get_number(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        self.0.usize(0..len)
    }
}
