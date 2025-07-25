//! Batcherbird Core Library
//! 
//! Core library for hardware synthesizer sampling automation.

pub mod error;
pub mod midi;
pub mod audio;
pub mod device;
pub mod session;
pub mod config;
pub mod sampler;
pub mod export;
pub mod detection;
pub mod loop_detection;

pub use error::{BatcherbirdError, Result};
pub use sampler::{AudioLevels, LevelMeterState};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}