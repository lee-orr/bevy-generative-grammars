use super::GrammarRandomNumberGenerator;

/// This is a wrapper struct that allows use of any type that implements the `rand::Rng` trait
pub struct Rand<'a, T: rand::Rng>(&'a mut T);

impl<'a, T: rand::Rng> Rand<'a, T> {
    /// Wraps a type implementing `rand::Rng` for use with grammars
    pub fn new(rng: &'a mut T) -> Self {
        Self(rng)
    }
}

impl<'a, T: rand::Rng> GrammarRandomNumberGenerator for Rand<'a, T> {
    fn get_number(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        self.0.gen_range(0..len)
    }
}

/// This is a wrapper struct that allows use of any type that implements the `rand::Rng` trait
/// Unlike Rand - this takes over the original object.
pub struct RandOwned<T: rand::Rng>(T);

impl<T: rand::Rng> RandOwned<T> {
    /// Wraps a type implementing `rand::Rng` for use with grammars
    pub fn new(rng: T) -> Self {
        Self(rng)
    }
}

impl<T: rand::Rng> GrammarRandomNumberGenerator for RandOwned<T> {
    fn get_number(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        self.0.gen_range(0..len)
    }
}
