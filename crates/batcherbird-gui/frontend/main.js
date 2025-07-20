const { invoke } = window.__TAURI__.core;

let selectedMidiDevice = '';
let selectedAudioInputDevice = '';
let selectedAudioOutputDevice = '';

async function loadMidiDevices() {
    const select = document.getElementById('midi-select');
    const status = document.getElementById('status');
    
    console.log('Loading MIDI devices...');
    
    try {
        select.innerHTML = '<option value="">Loading...</option>';
        console.log('Calling invoke...');
        const devices = await invoke('list_midi_devices');
        console.log('Got devices:', devices);
        
        select.innerHTML = '<option value="">Select MIDI device...</option>';
        devices.forEach((device, index) => {
            const option = document.createElement('option');
            option.value = index;
            option.textContent = device;
            
            // Auto-select MiniFuse if available
            if (device.includes('MiniFuse')) {
                option.selected = true;
                selectedMidiDevice = device;
            }
            
            select.appendChild(option);
        });
        
        showStatus(`Found ${devices.length} MIDI devices`, 'success');
    } catch (error) {
        console.error('MIDI devices error:', error);
        select.innerHTML = '<option value="">Error loading devices</option>';
        showStatus(`Error loading MIDI devices: ${error}`, 'error');
    }
}

async function loadAudioInputDevices() {
    const select = document.getElementById('audio-input-select');
    
    try {
        select.innerHTML = '<option value="">Loading...</option>';
        const devices = await invoke('list_audio_input_devices');
        
        select.innerHTML = '<option value="">Select audio input device...</option>';
        devices.forEach((device, index) => {
            const option = document.createElement('option');
            option.value = index;
            option.textContent = device;
            
            // Auto-select MiniFuse if available
            if (device.includes('MiniFuse')) {
                option.selected = true;
                selectedAudioInputDevice = device;
            }
            
            select.appendChild(option);
        });
        
        showStatus(`Found ${devices.length} audio input devices`, 'success');
    } catch (error) {
        select.innerHTML = '<option value="">Error loading devices</option>';
        showStatus(`Error loading audio input devices: ${error}`, 'error');
    }
}

async function loadAudioOutputDevices() {
    const select = document.getElementById('audio-output-select');
    
    try {
        select.innerHTML = '<option value="">Loading...</option>';
        const devices = await invoke('list_audio_output_devices');
        
        select.innerHTML = '<option value="">Select audio output device...</option>';
        devices.forEach((device, index) => {
            const option = document.createElement('option');
            option.value = index;
            option.textContent = device;
            
            // Auto-select speakers or MiniFuse
            if (device.includes('MacBook') || device.includes('Built-in') || device.includes('MiniFuse')) {
                option.selected = true;
                selectedAudioOutputDevice = device;
            }
            
            select.appendChild(option);
        });
        
        showStatus(`Found ${devices.length} audio output devices`, 'success');
    } catch (error) {
        select.innerHTML = '<option value="">Error loading devices</option>';
        showStatus(`Error loading audio output devices: ${error}`, 'error');
    }
}

function showStatus(message, type) {
    const status = document.getElementById('status');
    status.textContent = message;
    status.className = `status ${type}`;
    status.style.display = 'block';
    
    // Hide status after 3 seconds for success messages
    if (type === 'success') {
        setTimeout(() => {
            status.style.display = 'none';
        }, 3000);
    }
}

// Handle device selection changes
document.getElementById('midi-select').addEventListener('change', async function(e) {
    const selectedIndex = e.target.value;
    if (selectedIndex !== '') {
        selectedMidiDevice = e.target.options[e.target.selectedIndex].textContent;
        
        try {
            const result = await invoke('connect_midi_device', { deviceIndex: parseInt(selectedIndex) });
            showStatus(`Connected to MIDI: ${selectedMidiDevice}`, 'success');
        } catch (error) {
            showStatus(`Failed to connect to MIDI device: ${error}`, 'error');
        }
    }
});

document.getElementById('audio-input-select').addEventListener('change', function(e) {
    const selectedIndex = e.target.value;
    if (selectedIndex !== '') {
        selectedAudioInputDevice = e.target.options[e.target.selectedIndex].textContent;
        showStatus(`Selected audio input: ${selectedAudioInputDevice}`, 'success');
    }
});

document.getElementById('audio-output-select').addEventListener('change', function(e) {
    const selectedIndex = e.target.value;
    if (selectedIndex !== '') {
        selectedAudioOutputDevice = e.target.options[e.target.selectedIndex].textContent;
        showStatus(`Selected audio output: ${selectedAudioOutputDevice}`, 'success');
    }
});

// Load devices when page loads
window.addEventListener('DOMContentLoaded', () => {
    loadMidiDevices();
    loadAudioInputDevices();
    loadAudioOutputDevices();
});

async function testMidiConnection() {
    try {
        const result = await invoke('test_midi_connection');
        showStatus(`MIDI Test: ${result}`, 'success');
    } catch (error) {
        showStatus(`MIDI Test Failed: ${error}`, 'error');
    }
}

// Make functions globally available
window.loadMidiDevices = loadMidiDevices;
window.loadAudioInputDevices = loadAudioInputDevices;
window.loadAudioOutputDevices = loadAudioOutputDevices;
window.testMidiConnection = testMidiConnection;