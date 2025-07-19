use crate::{Result, BatcherbirdError};
use cpal::{Device, Host, SupportedStreamConfig, traits::{DeviceTrait, HostTrait}};

pub struct AudioManager {
    host: Host,
}

impl AudioManager {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        Ok(Self { host })
    }

    pub fn list_input_devices(&self) -> Result<Vec<String>> {
        let mut devices = Vec::new();
        
        let input_devices = self.host.input_devices()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to enumerate input devices: {}", e)))?;
            
        for device in input_devices {
            if let Ok(name) = device.name() {
                devices.push(name);
            }
        }
        
        Ok(devices)
    }

    pub fn list_output_devices(&self) -> Result<Vec<String>> {
        let mut devices = Vec::new();
        
        let output_devices = self.host.output_devices()
            .map_err(|e| BatcherbirdError::Audio(format!("Failed to enumerate output devices: {}", e)))?;
            
        for device in output_devices {
            if let Ok(name) = device.name() {
                devices.push(name);
            }
        }
        
        Ok(devices)
    }
}