use crate::{Result, BatcherbirdError};
use crate::midi::MidiManager;
use crate::audio::AudioManager;
use midir::MidiOutputConnection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    MidiOutput,
    AudioInput,
    AudioOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    pub midi_output: Option<DeviceInfo>,
    pub audio_input: Option<DeviceInfo>,
    pub audio_output: Option<DeviceInfo>,
}

pub struct DeviceManager {
    midi_manager: MidiManager,
    audio_manager: AudioManager,
    midi_connection: Option<MidiOutputConnection>,
    current_state: DeviceState,
}

impl DeviceManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            midi_manager: MidiManager::new()?,
            audio_manager: AudioManager::new()?,
            midi_connection: None,
            current_state: DeviceState {
                midi_output: None,
                audio_input: None,
                audio_output: None,
            },
        })
    }

    pub fn list_midi_output_devices(&mut self) -> Result<Vec<DeviceInfo>> {
        let device_names = self.midi_manager.list_output_devices()?;
        let mut devices = Vec::new();
        
        for (index, name) in device_names.iter().enumerate() {
            devices.push(DeviceInfo {
                id: format!("midi_out_{}", index),
                name: name.clone(),
                device_type: DeviceType::MidiOutput,
                is_connected: self.current_state.midi_output
                    .as_ref()
                    .map(|d| d.name == *name)
                    .unwrap_or(false),
            });
        }
        
        Ok(devices)
    }

    pub fn list_audio_input_devices(&mut self) -> Result<Vec<DeviceInfo>> {
        let device_names = self.audio_manager.list_input_devices()?;
        let mut devices = Vec::new();
        
        for (index, name) in device_names.iter().enumerate() {
            devices.push(DeviceInfo {
                id: format!("audio_in_{}", index),
                name: name.clone(),
                device_type: DeviceType::AudioInput,
                is_connected: self.current_state.audio_input
                    .as_ref()
                    .map(|d| d.name == *name)
                    .unwrap_or(false),
            });
        }
        
        Ok(devices)
    }

    pub fn list_audio_output_devices(&mut self) -> Result<Vec<DeviceInfo>> {
        let device_names = self.audio_manager.list_output_devices()?;
        let mut devices = Vec::new();
        
        for (index, name) in device_names.iter().enumerate() {
            devices.push(DeviceInfo {
                id: format!("audio_out_{}", index),
                name: name.clone(),
                device_type: DeviceType::AudioOutput,
                is_connected: self.current_state.audio_output
                    .as_ref()
                    .map(|d| d.name == *name)
                    .unwrap_or(false),
            });
        }
        
        Ok(devices)
    }

    pub fn connect_midi_output(&mut self, device_index: usize) -> Result<()> {
        let connection = self.midi_manager.connect_output(device_index)?;
        
        // Get device name for state tracking
        let devices = self.midi_manager.list_output_devices()?;
        let device_name = devices.get(device_index)
            .ok_or_else(|| BatcherbirdError::Session("Invalid device index".to_string()))?;
        
        self.midi_connection = Some(connection);
        self.current_state.midi_output = Some(DeviceInfo {
            id: format!("midi_out_{}", device_index),
            name: device_name.clone(),
            device_type: DeviceType::MidiOutput,
            is_connected: true,
        });
        
        Ok(())
    }

    pub fn connect_audio_input(&mut self, device_index: usize) -> Result<()> {
        // Get device name for state tracking
        let devices = self.audio_manager.list_input_devices()?;
        let device_name = devices.get(device_index)
            .ok_or_else(|| BatcherbirdError::Session("Invalid device index".to_string()))?;
        
        self.current_state.audio_input = Some(DeviceInfo {
            id: format!("audio_in_{}", device_index),
            name: device_name.clone(),
            device_type: DeviceType::AudioInput,
            is_connected: true,
        });
        
        Ok(())
    }

    pub fn connect_audio_output(&mut self, device_index: usize) -> Result<()> {
        // Get device name for state tracking
        let devices = self.audio_manager.list_output_devices()?;
        let device_name = devices.get(device_index)
            .ok_or_else(|| BatcherbirdError::Session("Invalid device index".to_string()))?;
        
        self.current_state.audio_output = Some(DeviceInfo {
            id: format!("audio_out_{}", device_index),
            name: device_name.clone(),
            device_type: DeviceType::AudioOutput,
            is_connected: true,
        });
        
        Ok(())
    }

    pub fn get_device_state(&self) -> &DeviceState {
        &self.current_state
    }

    pub fn get_midi_connection(&mut self) -> Option<&mut MidiOutputConnection> {
        self.midi_connection.as_mut()
    }

    pub fn disconnect_all(&mut self) {
        self.midi_connection = None;
        self.current_state = DeviceState {
            midi_output: None,
            audio_input: None,
            audio_output: None,
        };
    }
}