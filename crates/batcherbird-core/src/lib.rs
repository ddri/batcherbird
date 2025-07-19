//! Batcherbird Core Library
//! 
//! Core library for hardware synthesizer sampling automation.

pub mod error;
pub mod midi;
pub mod audio;
pub mod session;
pub mod config;

pub use error::{BatcherbirdError, Result};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}