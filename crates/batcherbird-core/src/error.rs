use thiserror::Error;

pub type Result<T> = std::result::Result<T, BatcherbirdError>;

#[derive(Error, Debug)]
pub enum BatcherbirdError {
    #[error("MIDI error: {0}")]
    Midi(#[from] midir::InitError),

    #[error("MIDI connection error: {0}")]
    MidiConnection(#[from] midir::ConnectError<midir::MidiInput>),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Sample processing error: {0}")]
    Processing(String),

    #[error("Export error: {0}")]
    Export(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Session error: {0}")]
    Session(String),
}
