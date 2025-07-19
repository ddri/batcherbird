use crate::{Result, BatcherbirdError};
use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::collections::HashMap;

pub struct MidiManager {
    input: Option<MidiInput>,
    output: Option<MidiOutput>,
}

impl MidiManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            input: None,
            output: None,
        })
    }

    pub fn list_input_devices(&mut self) -> Result<Vec<String>> {
        let midi_in = MidiInput::new("batcherbird-input")?;
        let ports = midi_in.ports();
        let mut devices = Vec::new();
        
        for port in &ports {
            if let Ok(name) = midi_in.port_name(port) {
                devices.push(name);
            }
        }
        
        self.input = Some(midi_in);
        Ok(devices)
    }

    pub fn list_output_devices(&mut self) -> Result<Vec<String>> {
        let midi_out = MidiOutput::new("batcherbird-output")?;
        let ports = midi_out.ports();
        let mut devices = Vec::new();
        
        for port in &ports {
            if let Ok(name) = midi_out.port_name(port) {
                devices.push(name);
            }
        }
        
        self.output = Some(midi_out);
        Ok(devices)
    }
}