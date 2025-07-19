use crate::{Result, BatcherbirdError};
use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

    pub fn connect_output(&mut self, device_index: usize) -> Result<MidiOutputConnection> {
        let midi_out = self.output.take().unwrap_or_else(|| {
            MidiOutput::new("batcherbird-output").expect("Failed to create MIDI output")
        });
        
        let ports = midi_out.ports();
        if device_index >= ports.len() {
            return Err(BatcherbirdError::Session(format!(
                "MIDI output device index {} out of range (0-{})",
                device_index,
                ports.len().saturating_sub(1)
            )));
        }
        
        let port = &ports[device_index];
        let device_name = midi_out.port_name(port)
            .unwrap_or_else(|_| format!("Device {}", device_index));
            
        let conn_out = midi_out.connect(port, &format!("batcherbird-out-{}", device_name))
            .map_err(|e| BatcherbirdError::Session(format!("Failed to connect to MIDI output: {:?}", e)))?;
            
        Ok(conn_out)
    }

    pub fn send_note_on(conn: &mut MidiOutputConnection, channel: u8, note: u8, velocity: u8) -> Result<()> {
        let msg = [0x90 | (channel & 0x0F), note & 0x7F, velocity & 0x7F];
        conn.send(&msg)
            .map_err(|e| BatcherbirdError::Session(format!("Failed to send note on: {:?}", e)))?;
        Ok(())
    }

    pub fn send_note_off(conn: &mut MidiOutputConnection, channel: u8, note: u8, velocity: u8) -> Result<()> {
        let msg = [0x80 | (channel & 0x0F), note & 0x7F, velocity & 0x7F];
        conn.send(&msg)
            .map_err(|e| BatcherbirdError::Session(format!("Failed to send note off: {:?}", e)))?;
        Ok(())
    }

    pub async fn send_test_note(conn: &mut MidiOutputConnection, channel: u8, note: u8, velocity: u8, duration: Duration) -> Result<()> {
        // Send note on
        Self::send_note_on(conn, channel, note, velocity)?;
        
        // Wait for specified duration
        tokio::time::sleep(duration).await;
        
        // Send note off
        Self::send_note_off(conn, channel, note, velocity)?;
        
        Ok(())
    }

    pub fn connect_input(&mut self, device_index: usize) -> Result<MidiInputConnection<()>> {
        let midi_in = self.input.take().unwrap_or_else(|| {
            MidiInput::new("batcherbird-input").expect("Failed to create MIDI input")
        });
        
        let ports = midi_in.ports();
        if device_index >= ports.len() {
            return Err(BatcherbirdError::Session(format!(
                "MIDI input device index {} out of range (0-{})",
                device_index,
                ports.len().saturating_sub(1)
            )));
        }
        
        let port = &ports[device_index];
        let device_name = midi_in.port_name(port)
            .unwrap_or_else(|_| format!("Device {}", device_index));
            
        let conn_in = midi_in.connect(port, &format!("batcherbird-in-{}", device_name), 
            move |timestamp, message, _| {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                    
                Self::print_midi_message(now, timestamp, message);
            }, ())
            .map_err(|e| BatcherbirdError::Session(format!("Failed to connect to MIDI input: {:?}", e)))?;
            
        Ok(conn_in)
    }

    fn print_midi_message(timestamp_ms: u128, midi_timestamp: u64, message: &[u8]) {
        if message.is_empty() {
            return;
        }

        let time_str = format!("{:02}:{:02}:{:02}.{:03}", 
            (timestamp_ms / 3600000) % 24,
            (timestamp_ms / 60000) % 60,
            (timestamp_ms / 1000) % 60,
            timestamp_ms % 1000
        );

        let status = message[0];
        let msg_type = status & 0xF0;
        let channel = (status & 0x0F) + 1;

        match msg_type {
            0x90 if message.len() >= 3 && message[2] > 0 => {
                let note = message[1];
                let velocity = message[2];
                let note_name = Self::note_to_name(note);
                println!("[{}] Note On  Ch:{} Note:{}({}) Vel:{}", 
                    time_str, channel, note, note_name, velocity);
            }
            0x80 | 0x90 if message.len() >= 3 => { // Note off or note on with vel 0
                let note = message[1];
                let velocity = message[2];
                let note_name = Self::note_to_name(note);
                println!("[{}] Note Off Ch:{} Note:{}({}) Vel:{}", 
                    time_str, channel, note, note_name, velocity);
            }
            0xB0 if message.len() >= 3 => {
                let controller = message[1];
                let value = message[2];
                println!("[{}] CC       Ch:{} CC:{} Val:{}", 
                    time_str, channel, controller, value);
            }
            0xC0 if message.len() >= 2 => {
                let program = message[1];
                println!("[{}] Program  Ch:{} Prog:{}", 
                    time_str, channel, program);
            }
            _ => {
                let hex_msg: Vec<String> = message.iter().map(|b| format!("{:02X}", b)).collect();
                println!("[{}] Raw      {}", time_str, hex_msg.join(" "));
            }
        }
    }

    fn note_to_name(note: u8) -> String {
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let octave = (note / 12).saturating_sub(1);
        let note_name = note_names[(note % 12) as usize];
        format!("{}{}", note_name, octave)
    }
}