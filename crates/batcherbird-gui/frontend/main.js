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
        
        console.log('Audio output devices found:', devices);
        
        select.innerHTML = '<option value="">Select audio output device...</option>';
        devices.forEach((device, index) => {
            console.log(`Device ${index}: ${device}`);
            const option = document.createElement('option');
            option.value = index;
            option.textContent = device;
            
            // Auto-select speakers or MiniFuse
            if (device.includes('MacBook') || device.includes('Built-in') || device.includes('MiniFuse') || device.includes('Speakers')) {
                option.selected = true;
                selectedAudioOutputDevice = device;
                console.log('Auto-selected:', device);
            }
            
            select.appendChild(option);
        });
        
        showStatus(`Found ${devices.length} audio output devices`, 'success');
    } catch (error) {
        console.error('Audio output devices error:', error);
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
    console.log('MIDI device selection changed:', e.target.value);
    const selectedIndex = e.target.value;
    if (selectedIndex !== '') {
        selectedMidiDevice = e.target.options[e.target.selectedIndex].textContent;
        console.log('Selected MIDI device:', selectedMidiDevice, 'at index:', selectedIndex);
        
        try {
            console.log('Attempting to connect to MIDI device...');
            const result = await invoke('connect_midi_device', { deviceIndex: parseInt(selectedIndex) });
            console.log('MIDI connection result:', result);
            showStatus(`Connected to MIDI: ${selectedMidiDevice}`, 'success');
            
            // Enable preview button now that MIDI is connected
            const previewBtn = document.getElementById('preview-btn');
            previewBtn.disabled = false;
            console.log('Preview button enabled:', !previewBtn.disabled);
        } catch (error) {
            console.error('MIDI connection failed:', error);
            showStatus(`Failed to connect to MIDI device: ${error}`, 'error');
        }
    }
});

// Event listeners will be attached after DOM loads

// Load devices when page loads
window.addEventListener('DOMContentLoaded', () => {
    console.log('DOM loaded, attaching event listeners...');
    
    // Audio input event listener
    const audioInputSelect = document.getElementById('audio-input-select');
    if (audioInputSelect) {
        console.log('✅ Adding event listener to audio-input-select');
        audioInputSelect.addEventListener('change', function(e) {
            console.log('Audio input device selection changed:', e.target.value);
            const selectedIndex = e.target.value;
            if (selectedIndex !== '') {
                selectedAudioInputDevice = e.target.options[e.target.selectedIndex].textContent;
                console.log('Selected audio input device:', selectedAudioInputDevice, 'at index:', selectedIndex);
                showStatus(`Selected audio input: ${selectedAudioInputDevice}`, 'success');
            }
        });
    } else {
        console.error('❌ Cannot add event listener - audio-input-select not found');
    }

    // Audio output event listener
    const audioOutputSelect = document.getElementById('audio-output-select');
    if (audioOutputSelect) {
        console.log('✅ Adding event listener to audio-output-select');
        audioOutputSelect.addEventListener('change', function(e) {
            console.log('Audio output device selection changed:', e.target.value);
            const selectedIndex = e.target.value;
            if (selectedIndex !== '') {
                selectedAudioOutputDevice = e.target.options[e.target.selectedIndex].textContent;
                console.log('Selected audio output device:', selectedAudioOutputDevice, 'at index:', selectedIndex);
                showStatus(`Selected audio output: ${selectedAudioOutputDevice}`, 'success');
            }
        });
    } else {
        console.error('❌ Cannot add event listener - audio-output-select not found');
    }
    
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

// Update display values for sliders
document.getElementById('velocity-input').addEventListener('input', function(e) {
    document.getElementById('velocity-display').textContent = e.target.value;
});

document.getElementById('duration-input').addEventListener('input', function(e) {
    document.getElementById('duration-display').textContent = e.target.value;
});

async function previewNote() {
    console.log('Preview note button clicked!');
    
    const note = parseInt(document.getElementById('note-select').value);
    const velocity = parseInt(document.getElementById('velocity-input').value);
    const duration = parseInt(document.getElementById('duration-input').value);
    
    console.log('Preview parameters:', { note, velocity, duration });
    
    try {
        console.log('Calling preview_note invoke...');
        const result = await invoke('preview_note', { 
            note: note, 
            velocity: velocity, 
            duration: duration 
        });
        console.log('Preview result:', result);
        showStatus(`${result} (Note: ${note}, Velocity: ${velocity}, Duration: ${duration}ms)`, 'success');
    } catch (error) {
        console.error('Preview error:', error);
        showStatus(`Preview failed: ${error}`, 'error');
    }
}

async function recordSample() {
    const note = parseInt(document.getElementById('note-select').value);
    const velocity = parseInt(document.getElementById('velocity-input').value);
    const duration = parseInt(document.getElementById('duration-input').value);
    
    const recordBtn = document.getElementById('record-btn');
    const recordingStatus = document.getElementById('recording-status');
    const progressFill = document.getElementById('progress-fill');
    const recordingText = document.getElementById('recording-text');
    
    try {
        // Disable record button and show recording status
        recordBtn.disabled = true;
        recordBtn.textContent = '⏹️ Recording...';
        recordingStatus.style.display = 'block';
        progressFill.style.width = '0%';
        recordingText.textContent = 'Recording...';
        
        // Simulate progress animation
        let progress = 0;
        const progressInterval = setInterval(() => {
            progress += 2;
            progressFill.style.width = `${Math.min(progress, 100)}%`;
        }, duration / 50);
        
        // Call backend recording function (to be implemented)
        // const result = await invoke('record_single_note', { note, velocity, duration });
        
        // For now, just simulate recording
        await new Promise(resolve => setTimeout(resolve, duration + 500));
        
        clearInterval(progressInterval);
        progressFill.style.width = '100%';
        recordingText.textContent = 'Recording complete!';
        
        showStatus(`Recorded note ${note} successfully!`, 'success');
        
        // Hide recording status after 2 seconds
        setTimeout(() => {
            recordingStatus.style.display = 'none';
        }, 2000);
        
    } catch (error) {
        showStatus(`Recording failed: ${error}`, 'error');
        recordingStatus.style.display = 'none';
    } finally {
        // Re-enable record button
        recordBtn.disabled = false;
        recordBtn.textContent = '🔴 Record Sample';
    }
}

// Make functions globally available
window.loadMidiDevices = loadMidiDevices;
window.loadAudioInputDevices = loadAudioInputDevices;
window.loadAudioOutputDevices = loadAudioOutputDevices;
window.testMidiConnection = testMidiConnection;
window.previewNote = previewNote;
window.recordSample = recordSample;