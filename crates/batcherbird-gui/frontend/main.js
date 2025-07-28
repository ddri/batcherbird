const { invoke } = window.__TAURI__.core;
const { convertFileSrc } = window.__TAURI__.core;

let selectedMidiDevice = '';
let selectedAudioInputDevice = '';
let selectedAudioOutputDevice = '';
let currentRecordingMode = 'single'; // 'single' or 'range'

// Preference keys for localStorage
const PREFS = {
    MIDI_DEVICE: 'batcherbird_midi_device',
    AUDIO_INPUT_DEVICE: 'batcherbird_audio_input_device',
    AUDIO_OUTPUT_DEVICE: 'batcherbird_audio_output_device',
    OUTPUT_DIRECTORY: 'batcherbird_output_directory',
    SAMPLE_NAME: 'batcherbird_sample_name',
    EXPORT_FORMAT: 'batcherbird_export_format',
    DETECTION_ENABLED: 'batcherbird_detection_enabled',
    DETECTION_PRESET: 'batcherbird_detection_preset',
    DETECTION_THRESHOLD: 'batcherbird_detection_threshold',
    VELOCITY_LAYERS_ENABLED: 'batcherbird_velocity_layers_enabled',
    VELOCITY_LAYERS_PRESET: 'batcherbird_velocity_layers_preset',
    VELOCITY_LAYERS_CUSTOM: 'batcherbird_velocity_layers_custom'
};

// Load saved preferences
function loadPreferences() {
    selectedMidiDevice = localStorage.getItem(PREFS.MIDI_DEVICE) || '';
    selectedAudioInputDevice = localStorage.getItem(PREFS.AUDIO_INPUT_DEVICE) || '';
    selectedAudioOutputDevice = localStorage.getItem(PREFS.AUDIO_OUTPUT_DEVICE) || '';
    
    // Load output directory preference
    const savedOutputDir = localStorage.getItem(PREFS.OUTPUT_DIRECTORY);
    if (savedOutputDir) {
        const outputDirInput = document.getElementById('output-directory');
        if (outputDirInput) {
            outputDirInput.value = savedOutputDir;
        }
    }
    
    // Load sample name preference
    const savedSampleName = localStorage.getItem(PREFS.SAMPLE_NAME);
    if (savedSampleName) {
        const sampleNameInput = document.getElementById('sample-name');
        if (sampleNameInput) {
            sampleNameInput.value = savedSampleName;
        }
    }
    
    // Load export format preference
    const savedExportFormat = localStorage.getItem(PREFS.EXPORT_FORMAT);
    if (savedExportFormat) {
        const exportFormatSelect = document.getElementById('export-format');
        if (exportFormatSelect) {
            exportFormatSelect.value = savedExportFormat;
        }
    }
    
    // Load detection preferences
    const detectionEnabled = localStorage.getItem(PREFS.DETECTION_ENABLED);
    const detectionPreset = localStorage.getItem(PREFS.DETECTION_PRESET);
    const detectionThreshold = localStorage.getItem(PREFS.DETECTION_THRESHOLD);
    
    const detectionEnabledCheckbox = document.getElementById('detection-enabled');
    const detectionPresetSelect = document.getElementById('detection-preset');
    const detectionThresholdSlider = document.getElementById('detection-threshold');
    const detectionThresholdDisplay = document.getElementById('detection-threshold-display');
    
    if (detectionEnabled !== null && detectionEnabledCheckbox) {
        detectionEnabledCheckbox.checked = detectionEnabled === 'true';
    }
    
    if (detectionPreset && detectionPresetSelect) {
        detectionPresetSelect.value = detectionPreset;
    }
    
    if (detectionThreshold && detectionThresholdSlider && detectionThresholdDisplay) {
        detectionThresholdSlider.value = detectionThreshold;
        detectionThresholdDisplay.textContent = detectionThreshold;
    }
    
    // Load velocity layers preferences
    const velocityLayersEnabled = localStorage.getItem(PREFS.VELOCITY_LAYERS_ENABLED);
    const velocityLayersPreset = localStorage.getItem(PREFS.VELOCITY_LAYERS_PRESET);
    const velocityLayersCustom = localStorage.getItem(PREFS.VELOCITY_LAYERS_CUSTOM);
    
    const velocityLayersEnabledCheckbox = document.getElementById('velocity-layers-enabled');
    const velocityLayersPresetSelect = document.getElementById('velocity-layers-preset');
    const velocityLayersCustomInput = document.getElementById('velocity-layers-custom');
    
    if (velocityLayersEnabled !== null && velocityLayersEnabledCheckbox) {
        velocityLayersEnabledCheckbox.checked = velocityLayersEnabled === 'true';
    }
    
    if (velocityLayersPreset && velocityLayersPresetSelect) {
        velocityLayersPresetSelect.value = velocityLayersPreset;
    }
    
    if (velocityLayersCustom && velocityLayersCustomInput) {
        velocityLayersCustomInput.value = velocityLayersCustom;
    }
    
    console.log('Loaded preferences:', { 
        selectedMidiDevice, selectedAudioInputDevice, selectedAudioOutputDevice, 
        outputDirectory: savedOutputDir,
        sampleName: savedSampleName,
        detectionEnabled, detectionPreset, detectionThreshold,
        velocityLayersEnabled, velocityLayersPreset, velocityLayersCustom
    });
}

// Save preferences
function savePreferences() {
    localStorage.setItem(PREFS.MIDI_DEVICE, selectedMidiDevice);
    localStorage.setItem(PREFS.AUDIO_INPUT_DEVICE, selectedAudioInputDevice);
    localStorage.setItem(PREFS.AUDIO_OUTPUT_DEVICE, selectedAudioOutputDevice);
    
    // Save output directory if changed
    const outputDirInput = document.getElementById('output-directory');
    if (outputDirInput) {
        localStorage.setItem(PREFS.OUTPUT_DIRECTORY, outputDirInput.value);
    }
    
    // Save sample name if changed
    const sampleNameInput = document.getElementById('sample-name');
    if (sampleNameInput) {
        localStorage.setItem(PREFS.SAMPLE_NAME, sampleNameInput.value);
    }
    
    // Save export format if changed
    const exportFormatSelect = document.getElementById('export-format');
    if (exportFormatSelect) {
        localStorage.setItem(PREFS.EXPORT_FORMAT, exportFormatSelect.value);
    }
    
    // Save detection preferences
    const detectionEnabledCheckbox = document.getElementById('detection-enabled');
    const detectionPresetSelect = document.getElementById('detection-preset');
    const detectionThresholdSlider = document.getElementById('detection-threshold');
    
    if (detectionEnabledCheckbox) {
        localStorage.setItem(PREFS.DETECTION_ENABLED, detectionEnabledCheckbox.checked.toString());
    }
    
    if (detectionPresetSelect) {
        localStorage.setItem(PREFS.DETECTION_PRESET, detectionPresetSelect.value);
    }
    
    if (detectionThresholdSlider) {
        localStorage.setItem(PREFS.DETECTION_THRESHOLD, detectionThresholdSlider.value);
    }
    
    // Save velocity layers preferences
    const velocityLayersEnabledCheckbox = document.getElementById('velocity-layers-enabled');
    const velocityLayersPresetSelect = document.getElementById('velocity-layers-preset');
    const velocityLayersCustomInput = document.getElementById('velocity-layers-custom');
    
    if (velocityLayersEnabledCheckbox) {
        localStorage.setItem(PREFS.VELOCITY_LAYERS_ENABLED, velocityLayersEnabledCheckbox.checked.toString());
    }
    
    if (velocityLayersPresetSelect) {
        localStorage.setItem(PREFS.VELOCITY_LAYERS_PRESET, velocityLayersPresetSelect.value);
    }
    
    if (velocityLayersCustomInput) {
        localStorage.setItem(PREFS.VELOCITY_LAYERS_CUSTOM, velocityLayersCustomInput.value);
    }
    
    console.log('Saved preferences:', { 
        selectedMidiDevice, selectedAudioInputDevice, selectedAudioOutputDevice, 
        outputDirectory: outputDirInput?.value,
        sampleName: sampleNameInput?.value,
        detectionEnabled: detectionEnabledCheckbox?.checked,
        detectionPreset: detectionPresetSelect?.value,
        detectionThreshold: detectionThresholdSlider?.value,
        velocityLayersEnabled: velocityLayersEnabledCheckbox?.checked,
        velocityLayersPreset: velocityLayersPresetSelect?.value,
        velocityLayersCustom: velocityLayersCustomInput?.value
    });
}

async function loadMidiDevices() {
    console.log('🔄 loadMidiDevices() called');
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
            
            // Auto-select based on saved preference or MiniFuse
            if ((selectedMidiDevice && device === selectedMidiDevice) || 
                (!selectedMidiDevice && device.includes('MiniFuse'))) {
                option.selected = true;
                selectedMidiDevice = device;
                
                // Auto-connect to this device
                setTimeout(async () => {
                    try {
                        console.log('Auto-connecting to MIDI device:', device, 'at index:', index);
                        const result = await invoke('connect_midi_device', { deviceIndex: index });
                        console.log('Auto-connection result:', result);
                        showStatus(`Auto-connected to MIDI: ${device}`, 'success');
                        
                        // Enable preview button
                        const previewBtn = document.getElementById('preview-btn');
                        previewBtn.disabled = false;
                        console.log('Preview button auto-enabled');
                    } catch (error) {
                        console.error('Auto-connection failed:', error);
                        showStatus(`Failed to auto-connect to MIDI: ${error}`, 'error');
                    }
                }, 500); // Small delay to ensure UI is ready
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
            
            // Auto-select based on saved preference or MiniFuse
            if ((selectedAudioInputDevice && device === selectedAudioInputDevice) || 
                (!selectedAudioInputDevice && device.includes('MiniFuse'))) {
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
            
            // Auto-select based on saved preference or fallback to speakers/MiniFuse
            if ((selectedAudioOutputDevice && device === selectedAudioOutputDevice) || 
                (!selectedAudioOutputDevice && (device.includes('MacBook') || device.includes('Built-in') || device.includes('MiniFuse') || device.includes('Speakers')))) {
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
    if (!status) {
        console.warn('⚠️ Status element not found, logging to console instead:', message);
        if (type === 'error') {
            console.error('🚨', message);
        } else if (type === 'success') {
            console.log('✅', message);
        } else {
            console.info('ℹ️', message);
        }
        return;
    }
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

// Event listeners will be attached after DOM loads

// Load devices when page loads
window.addEventListener('DOMContentLoaded', () => {
    console.log('DOM loaded, loading preferences and attaching event listeners...');
    
    // Load saved preferences first
    loadPreferences();
    
    // MIDI device selection event listener
    const midiSelect = document.getElementById('midi-select');
    if (midiSelect) {
        console.log('✅ Adding event listener to midi-select');
        midiSelect.addEventListener('change', async function(e) {
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
                    
                    // Save preference
                    savePreferences();
                    
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
    } else {
        console.error('❌ Cannot add event listener - midi-select not found');
    }
    
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
                savePreferences();
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
                savePreferences();
            }
        });
    } else {
        console.error('❌ Cannot add event listener - audio-output-select not found');
    }
    
    // Slider event listeners for single note
    const velocityInput = document.getElementById('velocity-input');
    const durationInput = document.getElementById('duration-input');
    
    if (velocityInput) {
        velocityInput.addEventListener('input', function(e) {
            document.getElementById('velocity-display').textContent = e.target.value;
        });
    }
    
    if (durationInput) {
        durationInput.addEventListener('input', function(e) {
            document.getElementById('duration-display').textContent = e.target.value;
        });
    }
    
    // Slider event listeners for range sampling
    const rangeVelocityInput = document.getElementById('range-velocity-input');
    const rangeDurationInput = document.getElementById('range-duration-input');
    
    if (rangeVelocityInput) {
        rangeVelocityInput.addEventListener('input', function(e) {
            document.getElementById('range-velocity-display').textContent = e.target.value;
        });
    }
    
    if (rangeDurationInput) {
        rangeDurationInput.addEventListener('input', function(e) {
            document.getElementById('range-duration-display').textContent = e.target.value;
        });
    }
    
    // Velocity layers duration slider for layers mode
    const rangeDurationInputLayers = document.getElementById('range-duration-input-layers');
    if (rangeDurationInputLayers) {
        rangeDurationInputLayers.addEventListener('input', function(e) {
            document.getElementById('range-duration-display-layers').textContent = e.target.value;
        });
    }
    
    // Detection controls event listeners
    const detectionEnabledCheckbox = document.getElementById('detection-enabled');
    const detectionPresetSelect = document.getElementById('detection-preset');
    const detectionThresholdSlider = document.getElementById('detection-threshold');
    const detectionThresholdDisplay = document.getElementById('detection-threshold-display');
    
    if (detectionEnabledCheckbox) {
        detectionEnabledCheckbox.addEventListener('change', function(e) {
            console.log('Detection enabled changed:', e.target.checked);
            savePreferences();
        });
    }
    
    if (detectionPresetSelect) {
        detectionPresetSelect.addEventListener('change', function(e) {
            console.log('Detection preset changed:', e.target.value);
            
            // Update threshold based on preset
            const thresholdSlider = document.getElementById('detection-threshold');
            const thresholdDisplay = document.getElementById('detection-threshold-display');
            
            if (thresholdSlider && thresholdDisplay) {
                let newThreshold = -35; // default
                switch (e.target.value) {
                    case 'percussive':
                        newThreshold = -30;
                        break;
                    case 'sustained':
                        newThreshold = -50;
                        break;
                    case 'vintage_synth':
                        newThreshold = -35;
                        break;
                    case 'default':
                        newThreshold = -40;
                        break;
                }
                
                thresholdSlider.value = newThreshold;
                thresholdDisplay.textContent = newThreshold;
            }
            
            savePreferences();
        });
    }
    
    if (detectionThresholdSlider && detectionThresholdDisplay) {
        detectionThresholdSlider.addEventListener('input', function(e) {
            detectionThresholdDisplay.textContent = e.target.value;
            savePreferences();
        });
    }
    
    // Velocity layers controls event listeners
    const velocityLayersEnabledCheckbox = document.getElementById('velocity-layers-enabled');
    const velocityLayersPresetSelect = document.getElementById('velocity-layers-preset');
    const velocityLayersCustomInput = document.getElementById('velocity-layers-custom');
    const singleVelocityRow = document.getElementById('single-velocity-row');
    const velocityLayersRow = document.getElementById('velocity-layers-row');
    
    // Function to toggle velocity layers UI
    function toggleVelocityLayersUI() {
        const isEnabled = velocityLayersEnabledCheckbox?.checked || false;
        const preset = velocityLayersPresetSelect?.value || '2';
        
        // Enable/disable preset selector
        if (velocityLayersPresetSelect) {
            velocityLayersPresetSelect.disabled = !isEnabled;
        }
        
        // Show/hide appropriate rows
        if (isEnabled) {
            singleVelocityRow.style.display = 'none';
            velocityLayersRow.style.display = 'flex';
            
            // Enable/disable custom input based on preset
            if (velocityLayersCustomInput) {
                velocityLayersCustomInput.disabled = preset !== 'custom';
                
                // Auto-populate based on preset
                if (preset !== 'custom' && preset !== velocityLayersCustomInput.value) {
                    switch (preset) {
                        case '2': velocityLayersCustomInput.value = '64,127'; break;
                        case '3': velocityLayersCustomInput.value = '48,96,127'; break;
                        case '4': velocityLayersCustomInput.value = '32,64,96,127'; break;
                    }
                }
            }
        } else {
            singleVelocityRow.style.display = 'flex';
            velocityLayersRow.style.display = 'none';
            
            if (velocityLayersCustomInput) {
                velocityLayersCustomInput.disabled = true;
            }
        }
    }
    
    // Set up initial state
    toggleVelocityLayersUI();
    
    // Initialize filename example
    updateFilenameExample();
    
    if (velocityLayersEnabledCheckbox) {
        velocityLayersEnabledCheckbox.addEventListener('change', function(e) {
            console.log('Velocity layers enabled changed:', e.target.checked);
            toggleVelocityLayersUI();
            savePreferences();
        });
    }
    
    if (velocityLayersPresetSelect) {
        velocityLayersPresetSelect.addEventListener('change', function(e) {
            console.log('Velocity layers preset changed:', e.target.value);
            toggleVelocityLayersUI();
            savePreferences();
        });
    }
    
    if (velocityLayersCustomInput) {
        velocityLayersCustomInput.addEventListener('input', function(e) {
            console.log('Custom velocities changed:', e.target.value);
            savePreferences();
        });
    }
    
    // Enable range preview when MIDI is connected
    const previewBtn = document.getElementById('preview-btn');
    const rangePreviewBtn = document.getElementById('range-preview-btn');
    if (previewBtn && rangePreviewBtn) {
        // When single note preview is enabled, enable range preview too
        const observer = new MutationObserver(() => {
            if (!previewBtn.disabled) {
                rangePreviewBtn.disabled = false;
            }
        });
        observer.observe(previewBtn, { attributes: true, attributeFilter: ['disabled'] });
    }
    
    // Load devices after DOM is ready
    loadMidiDevices();
    loadAudioInputDevices();
    loadAudioOutputDevices();
});

async function testMidiConnection() {
    console.log('🧪 testMidiConnection() called');
    try {
        const result = await invoke('test_midi_connection');
        showStatus(`MIDI Test: ${result}`, 'success');
    } catch (error) {
        showStatus(`MIDI Test Failed: ${error}`, 'error');
    }
}

// Slider event listeners will be added in DOMContentLoaded

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
    console.log('🔴 recordSample() called - starting recording process...');
    
    const note = parseInt(document.getElementById('note-select').value);
    const velocity = parseInt(document.getElementById('velocity-input').value);
    const duration = parseInt(document.getElementById('duration-input').value);
    
    console.log('Recording parameters:', { note, velocity, duration });
    
    const recordBtn = document.getElementById('record-btn');
    const recordingStatus = document.getElementById('recording-status');
    const progressFill = document.getElementById('progress-fill');
    const recordingText = document.getElementById('recording-text');
    
    try {
        // Disable record button and show recording status
        recordBtn.disabled = true;
        recordBtn.textContent = 'Recording...';
        recordingStatus.style.display = 'block';
        progressFill.style.width = '0%';
        recordingText.textContent = 'Starting recording...';
        
        // Animate progress bar
        let progress = 0;
        const progressInterval = setInterval(() => {
            progress += 2;
            progressFill.style.width = `${Math.min(progress, 100)}%`;
        }, duration / 50);
        
        console.log('Calling record_sample invoke...');
        
        // Get the output directory and sample name from the input fields
        const outputDirInput = document.getElementById('output-directory');
        const sampleNameInput = document.getElementById('sample-name');
        const outputDirectory = outputDirInput ? outputDirInput.value : './samples';
        const sampleName = sampleNameInput ? sampleNameInput.value.trim() : '';
        
        // Call the actual recording function
        // Get export format from settings
        const exportFormat = document.getElementById('export-format')?.value || 'wav';
        
        // Get Decent Sampler metadata if relevant
        const creatorName = document.getElementById('creator-name')?.value?.trim() || '';
        const instrumentDescription = document.getElementById('instrument-description')?.value?.trim() || '';
        
        console.log('📡 Calling backend record_sample with params:', { note, velocity, duration, outputDirectory, sampleName, exportFormat, creatorName, instrumentDescription });
        
        const result = await invoke('record_sample', { 
            note: note, 
            velocity: velocity, 
            duration: duration,
            outputDirectory: outputDirectory,
            sampleName: sampleName || null,
            exportFormat: exportFormat,
            creatorName: creatorName || null,
            instrumentDescription: instrumentDescription || null
        });
        console.log('✅ Backend returned result:', result);
        
        // Update UI with success
        recordingText.textContent = 'Recording complete!';
        showStatus(result, 'success');
        
        // Extract file path from result message and show waveform (async, non-blocking)
        setTimeout(async () => {
            try {
                // Parse the file path from the result message
                let filePath = null;
                
                // Try: "Location: /path/to/file"
                const locationMatch = result.match(/Location: (.+)/);
                if (locationMatch) {
                    filePath = locationMatch[1].trim();
                } else {
                    // Fallback: try to find any .wav file path in the result
                    const wavMatch = result.match(/([^\s]+\.wav)/);
                    if (wavMatch) {
                        filePath = wavMatch[1];
                    }
                }
                
                if (filePath) {
                    console.log('🌊 Updating main waveform for recorded file:', filePath);
                    await updateMainWaveform(filePath);
                } else {
                    console.log('ℹ️ Could not extract file path for waveform display');
                }
            } catch (waveformError) {
                console.error('❌ Failed to show waveform (non-critical):', waveformError);
            }
        }, 1000); // Delay to let file system sync
        
        clearInterval(progressInterval);
        progressFill.style.width = '100%';
        
        // Hide recording status after 3 seconds
        setTimeout(() => {
            recordingStatus.style.display = 'none';
        }, 3000);
        
    } catch (error) {
        console.error('Recording error:', error);
        showStatus(`Recording failed: ${error}`, 'error');
        recordingStatus.style.display = 'none';
    } finally {
        // Re-enable record button
        recordBtn.disabled = false;
        recordBtn.textContent = 'Record Sample';
    }
}

async function selectOutputDirectory() {
    console.log('📁 selectOutputDirectory() called - opening native macOS picker');
    try {
        const result = await invoke('select_output_directory');
        if (result) {
            const outputDirInput = document.getElementById('output-directory');
            outputDirInput.value = result;
            savePreferences();
            showStatus(`Output directory set to: ${result}`, 'success');
            console.log('✅ Directory selected via native picker:', result);
        }
    } catch (error) {
        if (error.includes('cancelled')) {
            console.log('❌ User cancelled directory selection');
        } else {
            console.error('Directory selection failed:', error);
            showStatus(`Failed to select directory: ${error}`, 'error');
        }
    }
}

// Simple test function to verify buttons work
function testButtonsWork() {
    console.log('✅ Button test function called - buttons are working!');
    showStatus('Button test successful!', 'success');
}

// Show samples folder in Finder
async function showSamplesInFinder() {
    console.log('📁 showSamplesInFinder() called');
    try {
        const result = await invoke('show_samples_in_finder');
        console.log('✅ Opened samples folder:', result);
        showStatus(result, 'success');
    } catch (error) {
        console.error('Failed to open samples folder:', error);
        showStatus(`Failed to open samples folder: ${error}`, 'error');
    }
}

// Global variable to track range recording state
let isRangeRecording = false;
let rangeRecordingAbortController = null;

// Helper function to get velocity layers configuration
function getVelocityLayers() {
    const velocityLayersEnabled = document.getElementById('velocity-layers-enabled')?.checked || false;
    
    if (!velocityLayersEnabled) {
        const velocity = parseInt(document.getElementById('range-velocity-input').value);
        return [velocity]; // Single velocity
    }
    
    const preset = document.getElementById('velocity-layers-preset')?.value || '2';
    const customInput = document.getElementById('velocity-layers-custom')?.value || '';
    
    if (preset === 'custom') {
        // Parse custom velocities from comma-separated string
        const velocities = customInput.split(',')
            .map(v => parseInt(v.trim()))
            .filter(v => !isNaN(v) && v >= 1 && v <= 127);
        return velocities.length > 0 ? velocities : [127]; // Fallback to max velocity
    } else {
        // Use preset velocities
        switch (preset) {
            case '2': return [64, 127];
            case '3': return [48, 96, 127];
            case '4': return [32, 64, 96, 127];
            default: return [127];
        }
    }
}

// Helper function to get duration for velocity layers
function getDurationForVelocityLayers() {
    // In the new UI, we use a single duration input for both single and multi-velocity
    const durationInput = document.getElementById('range-duration-input');
    return durationInput ? parseInt(durationInput.value) : 2000; // Default to 2000ms if not found
}

// Range sampling functions - using individual record_sample calls for real progress
async function recordRange() {
    console.log('🎹 recordRange() called - starting range recording process...');
    
    // Prevent double-clicking or starting when already recording
    if (isRangeRecording) {
        console.log('⚠️ Range recording already in progress, ignoring duplicate call');
        return;
    }
    
    const startNote = parseInt(document.getElementById('start-note-select').value);
    const endNote = parseInt(document.getElementById('end-note-select').value);
    const velocities = getVelocityLayers();
    const duration = getDurationForVelocityLayers();
    
    console.log('Range recording parameters:', { startNote, endNote, velocities, duration });
    
    // Validate range
    if (startNote >= endNote) {
        showStatus('Error: Start note must be lower than end note', 'error');
        return;
    }
    
    // Validate velocities
    if (velocities.length === 0) {
        showStatus('Error: No valid velocities configured', 'error');
        return;
    }
    
    const totalNotes = endNote - startNote + 1;
    const totalSamples = totalNotes * velocities.length;
    console.log(`Recording ${totalNotes} notes with ${velocities.length} velocity layers (${totalSamples} total samples)`);
    
    const rangeRecordBtn = document.getElementById('range-record-btn');
    const rangeStopBtn = document.getElementById('range-stop-btn');
    const rangeRecordingStatus = document.getElementById('range-recording-status');
    const rangeProgressFill = document.getElementById('range-progress-fill');
    const rangeRecordingText = document.getElementById('range-recording-text');
    const rangeCurrentNote = document.getElementById('range-current-note');
    
    try {
        // Set recording state
        isRangeRecording = true;
        rangeRecordingAbortController = new AbortController();
        
        // Update UI - show stop button, hide record button
        rangeRecordBtn.style.display = 'none';
        rangeStopBtn.style.display = 'inline-block';
        rangeRecordingStatus.style.display = 'block';
        rangeProgressFill.style.width = '0%';
        rangeRecordingText.textContent = velocities.length > 1 
            ? `Recording ${totalNotes} notes × ${velocities.length} velocities...`
            : `Recording ${totalNotes} notes...`;
        rangeCurrentNote.textContent = `Starting range recording...`;
        
        // Hide previous range waveform if shown
        hideWaveform(true);
        
        // Show velocity info if using layers
        const rangeVelocityInfo = document.getElementById('range-velocity-info');
        if (velocities.length > 1) {
            rangeVelocityInfo.textContent = `Velocity layers: ${velocities.join(', ')}`;
        } else {
            rangeVelocityInfo.textContent = `Single velocity: ${velocities[0]}`;
        }
        
        console.log('✅ UI Updated: Record button hidden, Stop button shown, Status bar visible');
        console.log('🔍 Debug: rangeRecordBtn display:', rangeRecordBtn.style.display);
        console.log('🔍 Debug: rangeStopBtn display:', rangeStopBtn.style.display);
        
        // Force a repaint to ensure UI updates are visible
        rangeStopBtn.offsetHeight;
        
        // Get the output directory and sample name from the input fields
        const outputDirInput = document.getElementById('output-directory');
        const sampleNameInput = document.getElementById('sample-name');
        const outputDirectory = outputDirInput ? outputDirInput.value : '';
        const sampleName = sampleNameInput ? sampleNameInput.value.trim() : '';
        
        // Function to convert MIDI note to name
        const noteToName = (note) => {
            const noteNames = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
            const octave = Math.floor(note / 12) - 1;
            const noteName = noteNames[note % 12];
            return `${noteName}${octave}`;
        };
        
        // Record each note individually for real progress AND working stop functionality
        console.log('📡 Starting individual note recording loop with real stop capability...');
        
        let successfulRecordings = 0;
        
        // Record each note individually for real progress AND working stop functionality
        console.log('📡 Starting individual note recording loop with real stop capability...');
        
        for (let i = 0; i < velocities.length; i++) {
            const velocity = velocities[i];
            
            if (rangeRecordingAbortController.signal.aborted) {
                console.log('⚠️ Range recording aborted by user');
                break;
            }
            
            for (let currentNote = startNote; currentNote <= endNote; currentNote++) {
                if (rangeRecordingAbortController.signal.aborted) {
                    console.log('⚠️ Range recording aborted by user');
                    break;
                }
                
                const currentNoteName = noteToName(currentNote);
                const sampleIndex = (i * totalNotes) + (currentNote - startNote) + 1;
                
                // Update UI
                rangeCurrentNote.textContent = `${currentNoteName} (${sampleIndex}/${totalSamples})`;
                rangeVelocityInfo.textContent = velocities.length > 1 
                    ? `Velocity ${velocity} (${i + 1}/${velocities.length})`
                    : `Velocity ${velocity}`;
                
                const progress = ((sampleIndex - 1) / totalSamples) * 100;
                rangeProgressFill.style.width = `${progress}%`;
                
                try {
                    console.log(`📡 Recording note ${currentNote} (${currentNoteName}) at velocity ${velocity}...`);
                    
                    // Record individual sample (WAV only now)
                    const result = await invoke('record_sample', { 
                        note: currentNote, 
                        velocity: velocity, 
                        duration: duration,
                        outputDirectory: outputDirectory,
                        sampleName: sampleName || null,
                        exportFormat: 'wav24bit', // Always WAV for individual samples
                        creatorName: '',  // No metadata for individual WAV files
                        instrumentDescription: ''  // No metadata for individual WAV files
                    });
                    
                    console.log(`✅ Note ${currentNoteName} recorded successfully`);
                    successfulRecordings++;
                    
                } catch (error) {
                    console.error(`❌ Failed to record note ${currentNoteName}:`, error);
                    showStatus(`Error recording ${currentNoteName}: ${error}`, 'error');
                    // Continue with other notes
                }
            }
        }
        
        // Final UI update
        if (isRangeRecording) {
            rangeProgressFill.style.width = '100%';
            if (successfulRecordings === totalSamples) {
                rangeRecordingText.textContent = 'Range recording complete!';
                rangeCurrentNote.textContent = `✅ Completed ${successfulRecordings} of ${totalSamples} samples successfully`;
                showStatus(`Range recording complete! ${successfulRecordings} samples saved.`, 'success');
                
                // Generate instrument files (.dspreset/.sfz) from the recorded samples
                const exportFormat = document.getElementById('export-format')?.value;
                if (exportFormat && exportFormat !== 'wav') {
                    try {
                        console.log(`🎼 Generating ${exportFormat} instrument file from recorded samples...`);
                        rangeRecordingText.textContent = 'Generating instrument files...';
                        rangeCurrentNote.textContent = `Creating ${exportFormat} file...`;
                        
                        // Get creator info from metadata fields if they exist
                        const creatorNameInput = document.getElementById('creator-name');
                        const instrumentDescriptionInput = document.getElementById('instrument-description');
                        const creatorName = creatorNameInput ? creatorNameInput.value.trim() : '';
                        const instrumentDescription = instrumentDescriptionInput ? instrumentDescriptionInput.value.trim() : '';
                        
                        // Handle different export format cases
                        if (exportFormat === 'all') {
                            // Generate both SFZ and Decent Sampler files
                            let baseDirectory = outputDirectory;
                            if (!baseDirectory) {
                                baseDirectory = sampleName ? 
                                    `/Users/dryan/Desktop/Batcherbird Samples/${sampleName}` : 
                                    '/Users/dryan/Desktop/Batcherbird Samples';
                            }
                            
                            // Generate SFZ file
                            rangeCurrentNote.textContent = `Creating SFZ file...`;
                            const sfzResult = await invoke('generate_instrument_files', {
                                directory: baseDirectory,
                                exportFormat: 'sfz',
                                sampleName: sampleName || null,
                                creatorName: creatorName || null,
                                instrumentDescription: instrumentDescription || null
                            });
                            console.log(`✅ SFZ file generated: ${sfzResult}`);
                            
                            // Generate Decent Sampler file
                            rangeCurrentNote.textContent = `Creating Decent Sampler file...`;
                            const dsResult = await invoke('generate_instrument_files', {
                                directory: baseDirectory,
                                exportFormat: 'decentsampler',
                                sampleName: sampleName || null,
                                creatorName: creatorName || null,
                                instrumentDescription: instrumentDescription || null
                            });
                            console.log(`✅ Decent Sampler file generated: ${dsResult}`);
                            
                            rangeRecordingText.textContent = 'Range recording complete!';
                            rangeCurrentNote.textContent = `✅ Generated SFZ + Decent Sampler files with ${successfulRecordings} samples`;
                            showStatus(`Range recording complete! ${successfulRecordings} samples + SFZ + Decent Sampler files saved.`, 'success');
                            
                        } else {
                            // Single format generation
                            let backendFormat = exportFormat;
                            if (exportFormat === 'kontakt') {
                                backendFormat = 'sfz'; // Use SFZ as stepping stone to Kontakt
                            }
                            
                            // Build the correct directory path that matches where samples were actually saved
                            let targetDirectory = outputDirectory;
                            if (!targetDirectory) {
                                targetDirectory = sampleName ? 
                                    `/Users/dryan/Desktop/Batcherbird Samples/${sampleName}` : 
                                    '/Users/dryan/Desktop/Batcherbird Samples';
                            }
                            
                            const instrumentResult = await invoke('generate_instrument_files', {
                                directory: targetDirectory,
                                exportFormat: backendFormat,
                                sampleName: sampleName || null,
                                creatorName: creatorName || null,
                                instrumentDescription: instrumentDescription || null
                            });
                            
                            console.log(`✅ Instrument file generated: ${instrumentResult}`);
                            rangeRecordingText.textContent = 'Range recording complete!';
                            rangeCurrentNote.textContent = `✅ Generated ${exportFormat} file with ${successfulRecordings} samples`;
                            showStatus(`Range recording complete! ${successfulRecordings} samples + ${exportFormat} file saved.`, 'success');
                        }
                        
                    } catch (error) {
                        console.error(`❌ Failed to generate ${exportFormat} file:`, error);
                        rangeRecordingText.textContent = 'Range recording complete (instrument file failed)';
                        rangeCurrentNote.textContent = `⚠️ WAV files saved, but ${exportFormat} generation failed`;
                        showStatus(`Range recording complete! ${successfulRecordings} samples saved, but ${exportFormat} generation failed.`, 'warning');
                    }
                }
            } else {
                rangeRecordingText.textContent = 'Range recording finished with errors';
                rangeCurrentNote.textContent = `⚠️ Completed ${successfulRecordings} of ${totalSamples} samples`;
                showStatus(`Range recording finished: ${successfulRecordings} of ${totalSamples} samples saved.`, 'error');
            }
        }
        
        // Hide recording status after 5 seconds
        setTimeout(() => {
            if (!isRangeRecording) {
                rangeRecordingStatus.style.display = 'none';
            }
        }, 5000);
        
    } catch (error) {
        console.error('Range recording error:', error);
        if (isRangeRecording) {
            showStatus(`Range recording failed: ${error}`, 'error');
        }
    } finally {
        // Reset UI
        resetRangeRecordingUI();
    }
}

// Stop range recording function
function stopRangeRecording() {
    console.log('🛑 stopRangeRecording() called');
    
    if (!isRangeRecording) {
        console.log('No recording in progress');
        return;
    }
    
    // Set flag to stop recording
    isRangeRecording = false;
    
    // Abort the backend call if possible
    if (rangeRecordingAbortController) {
        rangeRecordingAbortController.abort();
    }
    
    // Update UI immediately
    const rangeRecordingText = document.getElementById('range-recording-text');
    const rangeCurrentNote = document.getElementById('range-current-note');
    
    rangeRecordingText.textContent = 'Stopping recording...';
    rangeCurrentNote.textContent = 'Recording cancelled by user';
    
    showStatus('Range recording stopped by user', 'error');
    
    // Reset UI after a short delay
    setTimeout(() => {
        resetRangeRecordingUI();
    }, 2000);
}

// Helper function to reset range recording UI
function resetRangeRecordingUI() {
    console.log('🔄 Resetting range recording UI');
    
    const rangeRecordBtn = document.getElementById('range-record-btn');
    const rangeStopBtn = document.getElementById('range-stop-btn');
    const rangeRecordingStatus = document.getElementById('range-recording-status');
    
    // Reset recording state
    isRangeRecording = false;
    rangeRecordingAbortController = null;
    
    // Show record button, hide stop button
    rangeRecordBtn.style.display = 'inline-block';
    rangeStopBtn.style.display = 'none';
    
    // Hide status bar
    rangeRecordingStatus.style.display = 'none';
}

async function previewRange() {
    console.log('🎵 previewRange() called');
    
    const startNote = parseInt(document.getElementById('start-note-select').value);
    const endNote = parseInt(document.getElementById('end-note-select').value);
    const velocity = parseInt(document.getElementById('range-velocity-input').value);
    const duration = parseInt(document.getElementById('range-duration-input').value);
    
    // Validate range
    if (startNote >= endNote) {
        showStatus('Error: Start note must be lower than end note', 'error');
        return;
    }
    
    console.log('Preview range parameters:', { startNote, endNote, velocity, duration });
    
    try {
        // Preview just the start and end notes
        console.log('Previewing start note:', startNote);
        await invoke('preview_note', { 
            note: startNote, 
            velocity: velocity, 
            duration: 1000  // Shorter duration for preview
        });
        
        // Wait a bit then preview end note
        setTimeout(async () => {
            console.log('Previewing end note:', endNote);
            try {
                await invoke('preview_note', { 
                    note: endNote, 
                    velocity: velocity, 
                    duration: 1000
                });
                showStatus(`Range preview: ${startNote} to ${endNote} (${endNote - startNote + 1} notes)`, 'success');
            } catch (error) {
                console.error('End note preview error:', error);
                showStatus(`End note preview failed: ${error}`, 'error');
            }
        }, 1500);
        
    } catch (error) {
        console.error('Start note preview error:', error);
        showStatus(`Range preview failed: ${error}`, 'error');
    }
}

// Event listener for manual directory input changes
document.addEventListener('DOMContentLoaded', () => {
    console.log('🚀 Initializing new UI layout');
    
    // Initialize status bar
    updateStatusBar();
    
    // Load saved preferences
    loadPreferences();
    
    // Setup form event listeners
    const outputDirInput = document.getElementById('output-directory');
    if (outputDirInput) {
        outputDirInput.addEventListener('change', () => {
            savePreferences();
            showStatus(`Output directory updated: ${outputDirInput.value}`, 'success');
        });
    }
    
    // Event listener for sample name changes
    const sampleNameInput = document.getElementById('sample-name');
    if (sampleNameInput) {
        sampleNameInput.addEventListener('input', () => {
            savePreferences();
            updateFilenameExample();
        });
    }
    
    // Event listener for export format changes
    const exportFormatSelect = document.getElementById('export-format');
    if (exportFormatSelect) {
        exportFormatSelect.addEventListener('change', () => {
            savePreferences();
            updateFilenameExample();
            toggleDecentSamplerOptions();
        });
    }
    
    // Setup value display updates for range inputs
    setupRangeInputs();
    
    // Setup velocity layer controls
    setupVelocityLayerControls();
    
    // Initialize Decent Sampler options visibility
    toggleDecentSamplerOptions();
    
    // Setup device selection listeners
    setupDeviceSelectionListeners();
    
    // Initialize with single recording mode
    switchRecordingMode('single');
    
    // Initialize the main waveform display
    initializeMainWaveform();
    
    // Populate template dropdown
    populateTemplatesDropdown();
    
    console.log('✅ UI initialization complete');
});

function setupRangeInputs() {
    // Velocity slider
    const velocityInput = document.getElementById('velocity-input');
    const velocityDisplay = document.getElementById('velocity-display');
    if (velocityInput && velocityDisplay) {
        velocityInput.addEventListener('input', () => {
            velocityDisplay.textContent = velocityInput.value;
        });
    }
    
    // Duration slider
    const durationInput = document.getElementById('duration-input');
    const durationDisplay = document.getElementById('duration-display');
    if (durationInput && durationDisplay) {
        durationInput.addEventListener('input', () => {
            durationDisplay.textContent = durationInput.value;
        });
    }
    
    // Range velocity slider
    const rangeVelocityInput = document.getElementById('range-velocity-input');
    const rangeVelocityDisplay = document.getElementById('range-velocity-display');
    if (rangeVelocityInput && rangeVelocityDisplay) {
        rangeVelocityInput.addEventListener('input', () => {
            rangeVelocityDisplay.textContent = rangeVelocityInput.value;
        });
    }
    
    // Range duration slider
    const rangeDurationInput = document.getElementById('range-duration-input');
    const rangeDurationDisplay = document.getElementById('range-duration-display');
    if (rangeDurationInput && rangeDurationDisplay) {
        rangeDurationInput.addEventListener('input', () => {
            rangeDurationDisplay.textContent = rangeDurationInput.value;
        });
    }
    
    // Detection threshold slider
    const detectionThresholdInput = document.getElementById('detection-threshold');
    const detectionThresholdDisplay = document.getElementById('detection-threshold-display');
    if (detectionThresholdInput && detectionThresholdDisplay) {
        detectionThresholdInput.addEventListener('input', () => {
            detectionThresholdDisplay.textContent = detectionThresholdInput.value;
        });
    }
}

// Toggle metadata options for formats that support it
function toggleDecentSamplerOptions() {
    const exportFormat = document.getElementById('export-format')?.value;
    const decentSamplerOptions = document.getElementById('decent-sampler-options');
    const decentSamplerDescription = document.getElementById('decent-sampler-description');
    
    if (decentSamplerOptions && decentSamplerDescription) {
        // Show metadata options for SFZ and Decent Sampler formats
        if (exportFormat === 'decentsampler' || exportFormat === 'sfz') {
            decentSamplerOptions.style.display = 'block';
            decentSamplerDescription.style.display = 'block';
        } else {
            decentSamplerOptions.style.display = 'none';
            decentSamplerDescription.style.display = 'none';
        }
    }
}

function setupVelocityLayerControls() {
    const velocityLayersEnabled = document.getElementById('velocity-layers-enabled');
    const velocityLayersPreset = document.getElementById('velocity-layers-preset');
    const customVelocityControls = document.getElementById('custom-velocity-controls');
    const singleVelocityControls = document.getElementById('single-velocity-controls');
    const customVelocityInput = document.getElementById('velocity-layers-custom');
    
    if (velocityLayersEnabled) {
        velocityLayersEnabled.addEventListener('change', () => {
            const isEnabled = velocityLayersEnabled.checked;
            
            if (velocityLayersPreset) {
                velocityLayersPreset.disabled = !isEnabled;
            }
            
            if (singleVelocityControls) {
                singleVelocityControls.style.display = isEnabled ? 'none' : 'block';
            }
            
            savePreferences();
        });
    }
    
    if (velocityLayersPreset) {
        velocityLayersPreset.addEventListener('change', () => {
            const isCustom = velocityLayersPreset.value === 'custom';
            
            if (customVelocityControls) {
                customVelocityControls.style.display = isCustom ? 'block' : 'none';
            }
            
            if (customVelocityInput) {
                customVelocityInput.disabled = !isCustom;
            }
            
            savePreferences();
        });
    }
}

function setupDeviceSelectionListeners() {
    // MIDI device selection
    const midiSelect = document.getElementById('midi-select');
    if (midiSelect) {
        midiSelect.addEventListener('change', async () => {
            if (midiSelect.value && midiSelect.value !== '') {
                const deviceIndex = parseInt(midiSelect.value);
                const deviceName = midiSelect.options[midiSelect.selectedIndex].text;
                
                console.log(`🎹 User selected MIDI device: ${deviceName} (index: ${deviceIndex})`);
                
                try {
                    await connectMidiDevice(deviceIndex);
                    selectedMidiDevice = deviceName;
                    savePreferences();
                    updateStatusBar();
                    showStatus(`Connected to MIDI device: ${deviceName}`, 'success');
                } catch (error) {
                    console.error('❌ Failed to connect MIDI device:', error);
                    showStatus(`Failed to connect MIDI device: ${error}`, 'error');
                }
            } else {
                updateStatusBar();
            }
        });
    }
    
    // Audio device selections update status bar and save preferences
    const audioInSelect = document.getElementById('audio-input-select');
    if (audioInSelect) {
        audioInSelect.addEventListener('change', () => {
            if (audioInSelect.value && audioInSelect.value !== '') {
                selectedAudioInputDevice = audioInSelect.options[audioInSelect.selectedIndex].text;
                savePreferences();
                showStatus(`Selected audio input: ${selectedAudioInputDevice}`, 'success');
            }
            updateStatusBar();
        });
    }
    
    const audioOutSelect = document.getElementById('audio-output-select');
    if (audioOutSelect) {
        audioOutSelect.addEventListener('change', () => {
            if (audioOutSelect.value && audioOutSelect.value !== '') {
                selectedAudioOutputDevice = audioOutSelect.options[audioOutSelect.selectedIndex].text;
                savePreferences();
                showStatus(`Selected audio output: ${selectedAudioOutputDevice}`, 'success');
            }
            updateStatusBar();
        });
    }
}

// Load devices on startup
document.addEventListener('DOMContentLoaded', () => {
    setTimeout(() => {
        loadMidiDevicesWithStatus();
        loadAudioInputDevicesWithStatus();
        loadAudioOutputDevicesWithStatus();
    }, 500);
});

// Function to update the filename example display
function updateFilenameExample() {
    const sampleNameInput = document.getElementById('sample-name');
    const exportFormatSelect = document.getElementById('export-format');
    const exampleSpan = document.querySelector('span[style*="font-size: 11px"]');
    
    if (sampleNameInput && exportFormatSelect && exampleSpan) {
        const sampleName = sampleNameInput.value.trim();
        const exportFormat = exportFormatSelect.value;
        
        let exampleText = '';
        
        if (sampleName) {
            switch (exportFormat) {
                case 'sfz':
                    exampleText = `Example: ${sampleName}/${sampleName}_C4_60_vel127.wav + .sfz`;
                    break;
                case 'kontakt':
                    exampleText = `Example: ${sampleName}/${sampleName}_C4_60_vel127.wav + .nki`;
                    break;
                case 'decentsampler':
                    exampleText = `Example: ${sampleName}/${sampleName}_C4_60_vel127.wav + .dspreset`;
                    break;
                case 'all':
                    exampleText = `Example: ${sampleName}/ + WAV/SFZ/NKI/DSPRESET files`;
                    break;
                default:
                    exampleText = `Example: ${sampleName}/${sampleName}_C4_60_vel127.wav`;
            }
        } else {
            exampleText = `Example: C4_60_vel127.wav`;
        }
        
        exampleSpan.textContent = exampleText;
    }
}

// Responsive UI recording function with velocity layers - keeps interface responsive during long operations
async function recordNotesWithVelocityLayersResponsiveUI(startNote, endNote, velocities, duration, outputDirectory, sampleName,
    successfulRecordings, totalSamples, rangeProgressFill, rangeRecordingText, rangeCurrentNote, rangeVelocityInfo, noteToName) {
    
    window.rangeRecordingResults = { successfulRecordings: 0 };
    
    return new Promise((resolve) => {
        let currentNote = startNote;
        let currentVelocityIndex = 0;
        let sampleCount = 0;
        
        // Async scheduler - records one sample then yields control back to UI
        async function recordNextSample() {
            // Check if recording was stopped or completed
            if (!isRangeRecording || currentNote > endNote) {
                console.log(`🏁 Recording loop finished. Total samples: ${window.rangeRecordingResults.successfulRecordings}`);
                resolve();
                return;
            }
            
            const velocity = velocities[currentVelocityIndex];
            const progress = (sampleCount / totalSamples) * 100;
            const noteName = noteToName(currentNote);
            
            // Update progress UI - this happens on main thread, keeping UI responsive
            rangeProgressFill.style.width = `${progress}%`;
            rangeRecordingText.textContent = `Recording sample ${sampleCount + 1} of ${totalSamples}`;
            rangeCurrentNote.textContent = `♪ ${noteName} (${currentNote})`;
            
            // Show current velocity info
            if (velocities.length > 1) {
                rangeVelocityInfo.textContent = `Velocity layer ${currentVelocityIndex + 1}/${velocities.length}: vel ${velocity}`;
            } else {
                rangeVelocityInfo.textContent = `Velocity: ${velocity}`;
            }
            
            console.log(`🎵 Recording sample ${sampleCount + 1}/${totalSamples}: ${noteName} (${currentNote}) vel ${velocity}`);
            
            try {
                // Get export format from settings
                const exportFormat = document.getElementById('export-format')?.value || 'wav';
                
                // Get Decent Sampler metadata if relevant
                const creatorName = document.getElementById('creator-name')?.value?.trim() || '';
                const instrumentDescription = document.getElementById('instrument-description')?.value?.trim() || '';
                
                // Record individual sample with specific velocity
                const result = await invoke('record_sample', { 
                    note: currentNote, 
                    velocity: velocity, 
                    duration: duration,
                    outputDirectory: outputDirectory,
                    sampleName: sampleName || null,
                    exportFormat: exportFormat,
                    creatorName: creatorName || null,
                    instrumentDescription: instrumentDescription || null
                });
                
                console.log(`✅ Sample ${noteName} vel ${velocity} recorded successfully: ${result}`);
                window.rangeRecordingResults.successfulRecordings++;
                
                // Update current note to show success
                rangeCurrentNote.textContent = `✅ ${noteName} (${currentNote}) vel ${velocity} recorded`;
                
                // Show waveform for the recorded sample
                try {
                    const locationMatch = result.match(/Location: (.+\.wav)/);
                    if (locationMatch) {
                        const filePath = locationMatch[1];
                        console.log('🌊 Updating main waveform for range sample:', filePath);
                        await updateMainWaveform(filePath);
                        
                        // Still show the range-specific waveform
                        await showWaveform(filePath, true); // true for range mode
                        
                        // Update range waveform info
                        const rangeWaveformNote = document.getElementById('range-waveform-note');
                        const rangeWaveformVelocity = document.getElementById('range-waveform-velocity');
                        const rangeSamplesCount = document.getElementById('range-samples-count');
                        
                        if (rangeWaveformNote) rangeWaveformNote.textContent = `Note: ${noteName} (${currentNote})`;
                        if (rangeWaveformVelocity) rangeWaveformVelocity.textContent = `Velocity: ${velocity}`;
                        if (rangeSamplesCount) {
                            rangeSamplesCount.textContent = `Sample: ${window.rangeRecordingResults.successfulRecordings}/${totalSamples}`;
                        }
                    }
                } catch (waveformError) {
                    console.error('❌ Failed to show range waveform:', waveformError);
                    // Don't stop recording if waveform fails
                }
                
            } catch (sampleError) {
                console.error(`❌ Failed to record sample ${noteName} (${currentNote}) vel ${velocity}:`, sampleError);
                
                // Update current note to show error
                rangeCurrentNote.textContent = `❌ ${noteName} (${currentNote}) vel ${velocity} failed: ${sampleError}`;
                
                // Brief pause to show error, then continue
                setTimeout(() => {
                    advanceToNextSample();
                    setTimeout(recordNextSample, 0);
                }, 1000);
                return;
            }
            
            sampleCount++;
            advanceToNextSample();
            
            // Yield control back to UI thread between samples (keeps UI responsive)
            setTimeout(recordNextSample, 200); // 200ms delay for hardware + UI responsiveness
        }
        
        // Helper function to advance to next sample (handles velocity layers logic)
        function advanceToNextSample() {
            currentVelocityIndex++;
            
            // If we've finished all velocities for this note, move to next note
            if (currentVelocityIndex >= velocities.length) {
                currentVelocityIndex = 0;
                currentNote++;
            }
        }
        
        // Start the async recording process
        recordNextSample();
    });
}

// MIDI Panic function - professional audio safety feature
async function sendMidiPanic() {
    console.log('🚨 MIDI Panic button clicked!');
    
    try {
        console.log('Calling send_midi_panic invoke...');
        const result = await invoke('send_midi_panic');
        console.log('✅ MIDI Panic result:', result);
        showStatus(result, 'success');
    } catch (error) {
        console.error('❌ MIDI Panic error:', error);
        showStatus(`MIDI Panic failed: ${error}`, 'error');
    }
}

// New UI Layout Functions
function openSetupModal() {
    console.log('🔧 Opening device setup modal');
    const setupModal = document.getElementById('setup-modal');
    if (setupModal) {
        setupModal.style.display = 'flex';
        // Load devices when modal opens
        loadMidiDevicesWithStatus();
        loadAudioInputDevicesWithStatus();
        loadAudioOutputDevicesWithStatus();
    }
}

function closeSetupModal() {
    console.log('✅ Closing device setup modal');
    const setupModal = document.getElementById('setup-modal');
    if (setupModal) {
        setupModal.style.display = 'none';
    }
    updateStatusBar();
}

function switchRecordingMode(mode) {
    console.log(`🔄 Switching recording mode to: ${mode}`);
    currentRecordingMode = mode;
    
    // Update tab appearance
    const singleTab = document.getElementById('single-mode-tab');
    const rangeTab = document.getElementById('range-mode-tab');
    const singleRecording = document.getElementById('single-recording');
    const rangeRecording = document.getElementById('range-recording');
    
    if (mode === 'single') {
        singleTab.classList.add('active');
        rangeTab.classList.remove('active');
        singleRecording.style.display = 'block';
        rangeRecording.style.display = 'none';
    } else {
        singleTab.classList.remove('active');
        rangeTab.classList.add('active');
        singleRecording.style.display = 'none';
        rangeRecording.style.display = 'block';
    }
}

function updateStatusBar() {
    console.log('🔄 Updating status bar indicators');
    
    // Update MIDI status
    const midiStatus = document.getElementById('midi-status');
    const midiDeviceName = document.getElementById('midi-device-name');
    const midiSelect = document.getElementById('midi-select');
    
    if (midiSelect && midiSelect.value && midiSelect.value !== '') {
        midiStatus.className = 'status-indicator status-connected';
        midiDeviceName.textContent = midiSelect.options[midiSelect.selectedIndex].text;
        
        // Enable preview and record buttons when MIDI is connected
        const previewBtn = document.getElementById('preview-btn');
        const recordBtn = document.getElementById('record-btn');
        const rangePreviewBtn = document.getElementById('range-preview-btn');
        const rangeRecordBtn = document.getElementById('range-record-btn');
        
        if (previewBtn) previewBtn.disabled = false;
        if (recordBtn) recordBtn.disabled = false;
        if (rangePreviewBtn) rangePreviewBtn.disabled = false;
        if (rangeRecordBtn) rangeRecordBtn.disabled = false;
    } else {
        midiStatus.className = 'status-indicator status-disconnected';
        midiDeviceName.textContent = 'No MIDI Device';
        
        // Disable preview and record buttons when MIDI is not connected
        const previewBtn = document.getElementById('preview-btn');
        const recordBtn = document.getElementById('record-btn');
        const rangePreviewBtn = document.getElementById('range-preview-btn');
        const rangeRecordBtn = document.getElementById('range-record-btn');
        
        if (previewBtn) previewBtn.disabled = true;
        if (recordBtn) recordBtn.disabled = true;
        if (rangePreviewBtn) rangePreviewBtn.disabled = true;
        if (rangeRecordBtn) rangeRecordBtn.disabled = true;
    }
    
    // Update Audio Input status
    const audioInStatus = document.getElementById('audio-in-status');
    const audioInDeviceName = document.getElementById('audio-in-device-name');
    const audioInSelect = document.getElementById('audio-input-select');
    
    if (audioInSelect && audioInSelect.value && audioInSelect.value !== '') {
        audioInStatus.className = 'status-indicator status-connected';
        audioInDeviceName.textContent = audioInSelect.options[audioInSelect.selectedIndex].text;
    } else {
        audioInStatus.className = 'status-indicator status-disconnected';
        audioInDeviceName.textContent = 'No Audio Input';
    }
    
    // Update Audio Output status
    const audioOutStatus = document.getElementById('audio-out-status');
    const audioOutDeviceName = document.getElementById('audio-out-device-name');
    const audioOutSelect = document.getElementById('audio-output-select');
    
    if (audioOutSelect && audioOutSelect.value && audioOutSelect.value !== '') {
        audioOutStatus.className = 'status-indicator status-connected';
        audioOutDeviceName.textContent = audioOutSelect.options[audioOutSelect.selectedIndex].text;
    } else {
        audioOutStatus.className = 'status-indicator status-disconnected';
        audioOutDeviceName.textContent = 'No Audio Output';
    }
}

// Connect to MIDI device function
async function connectMidiDevice(deviceIndex) {
    console.log(`🔌 Connecting to MIDI device index: ${deviceIndex}`);
    
    try {
        const result = await invoke('connect_midi_device', { deviceIndex: deviceIndex });
        console.log('✅ MIDI device connected:', result);
        return result;
    } catch (error) {
        console.error('❌ Failed to connect MIDI device:', error);
        throw error;
    }
}

// Enhanced device loading functions that update status bar
async function loadMidiDevicesWithStatus() {
    console.log('🎹 Loading MIDI devices with status update...');
    
    try {
        const devices = await invoke('list_midi_devices');
        console.log('🎹 MIDI devices received:', devices);
        
        const midiSelect = document.getElementById('midi-select');
        if (midiSelect) {
            // Clear existing options
            midiSelect.innerHTML = '';
            
            if (devices.length === 0) {
                midiSelect.innerHTML = '<option value="">No MIDI devices found</option>';
            } else {
                midiSelect.innerHTML = '<option value="">Select MIDI device...</option>';
                let hasSelectedDevice = false;
                devices.forEach((device, index) => {
                    const option = document.createElement('option');
                    option.value = index.toString();
                    option.textContent = device;
                    midiSelect.appendChild(option);
                    
                    // Auto-select if this was the previously selected device
                    if (device === selectedMidiDevice) {
                        option.selected = true;
                        connectMidiDevice(index).catch(console.error);
                        hasSelectedDevice = true;
                    }
                });
                
                // If no saved device, auto-select first available device
                if (!hasSelectedDevice && devices.length > 0) {
                    const firstOption = midiSelect.options[1]; // Skip "Select device..." option
                    if (firstOption) {
                        firstOption.selected = true;
                        selectedMidiDevice = devices[0];
                        connectMidiDevice(0).catch(console.error);
                        console.log('🔧 Auto-selected first MIDI device:', devices[0]);
                    }
                }
            }
        }
        
        updateStatusBar();
    } catch (error) {
        console.error('❌ Failed to load MIDI devices:', error);
        showStatus(`Failed to load MIDI devices: ${error}`, 'error');
    }
}

async function loadAudioInputDevicesWithStatus() {
    console.log('🎤 Loading audio input devices with status update...');
    
    try {
        const devices = await invoke('list_audio_input_devices');
        console.log('🎤 Audio input devices received:', devices);
        
        const audioInputSelect = document.getElementById('audio-input-select');
        if (audioInputSelect) {
            audioInputSelect.innerHTML = '';
            
            if (devices.length === 0) {
                audioInputSelect.innerHTML = '<option value="">No audio input devices found</option>';
            } else {
                audioInputSelect.innerHTML = '<option value="">Select audio input device...</option>';
                let hasSelectedAudioInput = false;
                devices.forEach((device, index) => {
                    const option = document.createElement('option');
                    option.value = index.toString();
                    option.textContent = device;
                    audioInputSelect.appendChild(option);
                    
                    if (device === selectedAudioInputDevice) {
                        option.selected = true;
                        hasSelectedAudioInput = true;
                    }
                });
                
                // Auto-select first audio input if no saved preference
                if (!hasSelectedAudioInput && devices.length > 0) {
                    const firstOption = audioInputSelect.options[1];
                    if (firstOption) {
                        firstOption.selected = true;
                        selectedAudioInputDevice = devices[0];
                        console.log('🔧 Auto-selected first audio input device:', devices[0]);
                    }
                }
            }
        }
        
        updateStatusBar();
    } catch (error) {
        console.error('❌ Failed to load audio input devices:', error);
        showStatus(`Failed to load audio input devices: ${error}`, 'error');
    }
}

async function loadAudioOutputDevicesWithStatus() {
    console.log('🔊 Loading audio output devices with status update...');
    
    try {
        const devices = await invoke('list_audio_output_devices');
        console.log('🔊 Audio output devices received:', devices);
        
        const audioOutputSelect = document.getElementById('audio-output-select');
        if (audioOutputSelect) {
            audioOutputSelect.innerHTML = '';
            
            if (devices.length === 0) {
                audioOutputSelect.innerHTML = '<option value="">No audio output devices found</option>';
            } else {
                audioOutputSelect.innerHTML = '<option value="">Select audio output device...</option>';
                devices.forEach((device, index) => {
                    const option = document.createElement('option');
                    option.value = index.toString();
                    option.textContent = device;
                    audioOutputSelect.appendChild(option);
                    
                    if (device === selectedAudioOutputDevice) {
                        option.selected = true;
                    }
                });
            }
        }
        
        updateStatusBar();
    } catch (error) {
        console.error('❌ Failed to load audio output devices:', error);
        showStatus(`Failed to load audio output devices: ${error}`, 'error');
    }
}

// Make functions globally available IMMEDIATELY
window.loadMidiDevices = loadMidiDevicesWithStatus;
window.loadAudioInputDevices = loadAudioInputDevicesWithStatus;
window.loadAudioOutputDevices = loadAudioOutputDevicesWithStatus;
window.connectMidiDevice = connectMidiDevice;
window.testMidiConnection = testMidiConnection;
window.previewNote = previewNote;
window.recordSample = recordSample;
window.recordRange = recordRange;
window.stopRangeRecording = stopRangeRecording;
window.previewRange = previewRange;
window.selectOutputDirectory = selectOutputDirectory;
window.testButtonsWork = testButtonsWork;
window.showSamplesInFinder = showSamplesInFinder;
window.testLoopDetection = testLoopDetection;
window.sendMidiPanic = sendMidiPanic;
window.openSetupModal = openSetupModal;
window.closeSetupModal = closeSetupModal;
window.switchRecordingMode = switchRecordingMode;
window.updateStatusBar = updateStatusBar;

// ============================================================================
// WAVEFORM VISUALIZATION SYSTEM
// ============================================================================

let wavesurferInstance = null;
let rangeWavesurferInstance = null;
let mainWavesurferInstance = null;
let currentSamplePath = null;

// Initialize Wavesurfer.js when needed
async function initializeWaveform(containerId) {
    console.log(`🌊 Initializing waveform in container: ${containerId}`);
    
    try {
        // Import Wavesurfer and Regions plugin dynamically
        const WaveSurfer = (await import('https://unpkg.com/wavesurfer.js@7/dist/wavesurfer.esm.js')).default;
        const RegionsPlugin = (await import('https://unpkg.com/wavesurfer.js@7/dist/plugins/regions.esm.js')).default;
        
        const container = document.getElementById(containerId);
        if (!container) {
            console.error(`❌ Waveform container not found: ${containerId}`);
            return null;
        }
        
        // Create regions plugin instance
        const regions = RegionsPlugin.create();
        
        const wavesurfer = WaveSurfer.create({
            container: container,
            waveColor: '#4682B4',
            progressColor: '#dc2626',
            backgroundColor: '#1e1e1e',
            height: 128,
            normalize: true,
            fillParent: true,
            responsive: true,
            plugins: [regions]
        });
        
        // Store regions plugin reference for later use
        wavesurfer.regionsPlugin = regions;
        
        console.log('✅ Wavesurfer instance created successfully');
        return wavesurfer;
        
    } catch (error) {
        console.error('❌ Failed to initialize Wavesurfer:', error);
        return null;
    }
}

// Show waveform for a recorded sample
async function showWaveform(samplePath, isRangeMode = false) {
    console.log(`🌊 Showing waveform for: ${samplePath}`);
    
    const containerId = isRangeMode ? 'range-waveform-display' : 'waveform-display';
    const containerElementId = isRangeMode ? 'range-waveform-container' : 'waveform-container';
    const containerElement = document.getElementById(containerElementId);
    
    console.log(`🔍 Looking for container: ${containerElementId}`);
    console.log(`🔍 Container found:`, !!containerElement);
    
    if (!containerElement) {
        console.error(`❌ Waveform container not found: ${containerElementId}`);
        return;
    }
    
    // Show loading state
    const displayElement = document.getElementById(containerId);
    console.log(`🔍 Display element found:`, !!displayElement);
    displayElement.innerHTML = '<div class="waveform-loading">Loading waveform...</div>';
    containerElement.style.display = 'block';
    console.log(`🔍 Container made visible`);
    
    try {
        // Clean the file path (remove file:// prefix if present)
        let cleanPath = samplePath;
        if (cleanPath.startsWith('file://')) {
            cleanPath = cleanPath.replace('file://', '');
        }
        
        console.log(`🔧 Converting file path for Tauri: ${cleanPath}`);
        
        // Use Tauri's convertFileSrc to get proper asset URL
        const audioFileUrl = convertFileSrc(cleanPath);
        console.log(`✅ Converted to asset URL: ${audioFileUrl}`);
        
        // Initialize wavesurfer instance if needed
        let wavesurfer = isRangeMode ? rangeWavesurferInstance : wavesurferInstance;
        
        if (!wavesurfer) {
            // Clear loading state first
            displayElement.innerHTML = '';
            
            wavesurfer = await initializeWaveform(containerId);
            if (!wavesurfer) {
                throw new Error('Failed to initialize Wavesurfer');
            }
            
            if (isRangeMode) {
                rangeWavesurferInstance = wavesurfer;
            } else {
                wavesurferInstance = wavesurfer;
            }
        }
        
        // Load the audio file using the proper asset URL
        console.log(`🌊 Loading waveform with asset URL: ${audioFileUrl}`);
        await wavesurfer.load(audioFileUrl);
        currentSamplePath = audioFileUrl;
        
        // Update info display
        updateWaveformInfo(wavesurfer, isRangeMode);
        
        console.log('✅ Waveform loaded successfully');
        
    } catch (error) {
        console.error('❌ Failed to load waveform:', error);
        displayElement.innerHTML = `<div class="waveform-loading">Failed to load waveform: ${error.message}</div>`;
    }
}

// Update waveform information display
function updateWaveformInfo(wavesurfer, isRangeMode = false) {
    const duration = wavesurfer.getDuration();
    
    if (isRangeMode) {
        const durationSpan = document.getElementById('range-waveform-note');
        if (durationSpan) {
            durationSpan.textContent = `Duration: ${duration.toFixed(2)}s`;
        }
    } else {
        const durationSpan = document.getElementById('waveform-duration');
        if (durationSpan) {
            durationSpan.textContent = `Duration: ${duration.toFixed(2)}s`;
        }
        
        // TODO: Add auto-detection boundary info
        const boundariesSpan = document.getElementById('waveform-boundaries');
        if (boundariesSpan) {
            boundariesSpan.textContent = `Auto-detected: Start 0.00s, End ${duration.toFixed(2)}s`;
        }
    }
}

// Waveform control functions
function zoomWaveform(factor) {
    if (wavesurferInstance) {
        wavesurferInstance.zoom(wavesurferInstance.options.minPxPerSec * factor);
        console.log(`🔍 Zoomed waveform by factor: ${factor}`);
    }
}

function playWaveform() {
    if (wavesurferInstance) {
        if (wavesurferInstance.isPlaying()) {
            wavesurferInstance.pause();
            document.getElementById('waveform-play').textContent = '▶️';
        } else {
            wavesurferInstance.play();
            document.getElementById('waveform-play').textContent = '⏸️';
        }
    }
}

function playRangeWaveform() {
    if (rangeWavesurferInstance) {
        if (rangeWavesurferInstance.isPlaying()) {
            rangeWavesurferInstance.pause();
            document.getElementById('range-waveform-play').textContent = '▶️';
        } else {
            rangeWavesurferInstance.play();
            document.getElementById('range-waveform-play').textContent = '⏸️';
        }
    }
}

function resetWaveformView() {
    if (wavesurferInstance) {
        wavesurferInstance.zoom(1);
        wavesurferInstance.seekTo(0);
        console.log('🔄 Reset waveform view');
    }
}

function showBatchThumbnails() {
    // TODO: Implement batch thumbnail view
    console.log('🖼️ Batch thumbnails feature - coming soon!');
    showStatus('Batch thumbnails view coming in next update!', 'success');
}

// Hide waveform display
function hideWaveform(isRangeMode = false) {
    const containerElement = document.getElementById(isRangeMode ? 'range-waveform-container' : 'waveform-container');
    if (containerElement) {
        containerElement.style.display = 'none';
    }
}

// ============================================================================
// LOOP MARKERS AND VISUAL EDITING - SampleRobot/Ableton Style
// ============================================================================

let currentLoopRegion = null;

// Add visual loop markers to waveform display
function addLoopMarkersToWaveform(wavesurfer, startTime, endTime, isEditable = true) {
    if (!wavesurfer || !wavesurfer.regionsPlugin) {
        console.warn('⚠️ Cannot add loop markers: no regions plugin available');
        return null;
    }
    
    // Remove existing loop region if present
    if (currentLoopRegion) {
        currentLoopRegion.remove();
        currentLoopRegion = null;
    }
    
    console.log(`🎯 Adding loop markers: ${startTime.toFixed(3)}s - ${endTime.toFixed(3)}s`);
    
    // Create loop region with visual styling
    currentLoopRegion = wavesurfer.regionsPlugin.addRegion({
        start: startTime,
        end: endTime,
        color: 'rgba(255, 215, 0, 0.3)', // Gold color with transparency
        drag: isEditable,
        resize: isEditable,
        label: 'Loop'
    });
    
    if (isEditable) {
        // Set up event handlers for region changes
        currentLoopRegion.on('update-end', () => {
            const newStart = currentLoopRegion.start;
            const newEnd = currentLoopRegion.end;
            console.log(`🔄 Loop region updated: ${newStart.toFixed(3)}s - ${newEnd.toFixed(3)}s`);
            
            // Update the stored loop data
            if (window.currentLoopData) {
                const sampleRate = window.currentLoopData.sampleRate;
                window.currentLoopData.startTime = newStart;
                window.currentLoopData.endTime = newEnd;
                window.currentLoopData.startSample = Math.round(newStart * sampleRate);
                window.currentLoopData.endSample = Math.round(newEnd * sampleRate);
                
                console.log('✅ Updated loop data:', window.currentLoopData);
                
                // Update UI to show the loop has been modified
                updateLoopModificationStatus(true);
            }
        });
        
        // Enable region click to play loop
        currentLoopRegion.on('click', (e) => {
            if (e.shiftKey) {
                // Shift+click plays the region in a loop (WaveSurfer.js feature)
                console.log('🔁 Playing loop region on repeat');
            } else {
                // Regular click plays the region once
                console.log('▶️ Playing loop region once');
            }
        });
    }
    
    return currentLoopRegion;
}

// Update UI to indicate loop has been modified
function updateLoopModificationStatus(isModified) {
    const acceptBtn = document.getElementById('accept-loop-btn');
    if (acceptBtn && isModified) {
        acceptBtn.textContent = '💾 Save Changes';
        acceptBtn.disabled = false;
        acceptBtn.classList.remove('btn-secondary');
        acceptBtn.classList.add('btn-success');
    }
}

// Clear all loop markers from waveform
function clearLoopMarkers(wavesurfer) {
    if (currentLoopRegion) {
        currentLoopRegion.remove();
        currentLoopRegion = null;
        console.log('🗑️ Cleared loop markers');
    }
}

// Initialize main waveform display on page load
async function initializeMainWaveform() {
    try {
        console.log('🌊 Initializing main waveform display');
        mainWavesurferInstance = await initializeWaveform('main-waveform-display');
        if (mainWavesurferInstance) {
            console.log('✅ Main waveform initialized successfully');
        }
    } catch (error) {
        console.error('❌ Failed to initialize main waveform:', error);
    }
}

// Update main waveform with new recording
async function updateMainWaveform(audioFilePath) {
    if (!mainWavesurferInstance) {
        console.log('🌊 Main waveform not initialized, creating...');
        await initializeMainWaveform();
    }
    
    if (mainWavesurferInstance && audioFilePath) {
        try {
            console.log('📁 Loading audio into main waveform:', audioFilePath);
            
            // Hide placeholder and show waveform
            const placeholder = document.querySelector('#main-waveform-display .waveform-placeholder');
            if (placeholder) {
                placeholder.style.display = 'none';
            }
            
            // Add has-waveform class for styling
            const display = document.getElementById('main-waveform-display');
            if (display) {
                display.classList.add('has-waveform');
            }
            
            // Load the audio file
            const audioUrl = convertFileSrc(audioFilePath);
            await mainWavesurferInstance.load(audioUrl);
            
            // Update info display
            const duration = mainWavesurferInstance.getDuration();
            const durationEl = document.getElementById('main-waveform-duration');
            const fileEl = document.getElementById('main-waveform-file');
            
            if (durationEl) {
                durationEl.textContent = `Duration: ${duration.toFixed(2)}s`;
            }
            if (fileEl) {
                const fileName = audioFilePath.split('/').pop();
                fileEl.textContent = fileName;
            }
            
            // Enable controls
            ['main-waveform-play', 'main-waveform-zoom-in', 'main-waveform-zoom-out', 'main-waveform-reset'].forEach(id => {
                const btn = document.getElementById(id);
                if (btn) btn.disabled = false;
            });
            
            console.log('✅ Main waveform updated successfully');
            
        } catch (error) {
            console.error('❌ Failed to update main waveform:', error);
        }
    }
}

// Main waveform control functions
function playMainWaveform() {
    if (mainWavesurferInstance) {
        if (mainWavesurferInstance.isPlaying()) {
            mainWavesurferInstance.pause();
            document.querySelector('#main-waveform-play span').textContent = 'Play';
        } else {
            mainWavesurferInstance.play();
            document.querySelector('#main-waveform-play span').textContent = 'Pause';
        }
    }
}

function zoomMainWaveform(factor) {
    if (mainWavesurferInstance) {
        mainWavesurferInstance.zoom(mainWavesurferInstance.options.minPxPerSec * factor);
    }
}

function resetMainWaveformView() {
    if (mainWavesurferInstance) {
        mainWavesurferInstance.zoom(1);
        mainWavesurferInstance.seekTo(0);
    }
}

// Initialize main waveform when page loads
document.addEventListener('DOMContentLoaded', () => {
    initializeMainWaveform();
});

// Export waveform functions to global scope
window.zoomWaveform = zoomWaveform;
window.playWaveform = playWaveform;
window.playRangeWaveform = playRangeWaveform;
window.resetWaveformView = resetWaveformView;
window.showBatchThumbnails = showBatchThumbnails;
window.showWaveform = showWaveform;
window.hideWaveform = hideWaveform;
window.addLoopMarkersToWaveform = addLoopMarkersToWaveform;
window.clearLoopMarkers = clearLoopMarkers;
window.playMainWaveform = playMainWaveform;
window.zoomMainWaveform = zoomMainWaveform;
window.resetMainWaveformView = resetMainWaveformView;

// ============================================================================
// REAL-TIME LEVEL METERS SYSTEM - 60 FPS Professional Audio Monitoring
// ============================================================================

let levelMeterUpdateInterval = null;
let isLevelMeterActive = false;
let isInputMonitoringEnabled = false;

// Professional level meter configuration
const LEVEL_METER_CONFIG = {
    updateIntervalMs: 100, // 10 FPS for development (less spam, still responsive)
    dbFloor: -60,          // Minimum dB level to display
    dbCeiling: 0,          // Maximum dB level (0 dBFS)
    peakHoldTimeMs: 1500,  // Peak hold duration
    peakDecayRate: 0.02    // Peak decay speed per frame
};

// Peak hold state for animation
let inputPeakHold = {
    level: LEVEL_METER_CONFIG.dbFloor,
    timestamp: 0,
    isDecaying: false
};

// AKAI-style input monitoring toggle function
async function toggleInputMonitoring() {
    const monitorBtn = document.getElementById('monitor-input-btn');
    
    if (!isInputMonitoringEnabled) {
        // Start monitoring
        try {
            monitorBtn.textContent = '⏳ Starting...';
            monitorBtn.disabled = true;
            
            await startInputMonitoring();
            monitorBtn.classList.add('active');
            monitorBtn.textContent = 'Monitoring...';
            console.log('🎛️ Input monitoring enabled (AKAI style)');
            
        } catch (error) {
            console.error('❌ Failed to start monitoring:', error);
            monitorBtn.textContent = '🎛️ Monitor Input';
        } finally {
            monitorBtn.disabled = false;
        }
    } else {
        // Stop monitoring  
        try {
            monitorBtn.textContent = '⏳ Stopping...';
            monitorBtn.disabled = true;
            
            await stopInputMonitoring();
            monitorBtn.classList.remove('active');
            monitorBtn.textContent = '🎛️ Monitor Input';
            console.log('🎛️ Input monitoring disabled');
            
        } catch (error) {
            console.error('❌ Failed to stop monitoring:', error);
        } finally {
            monitorBtn.disabled = false;
        }
    }
}

// Start input monitoring (professional sampler pattern)
async function startInputMonitoring() {
    console.log('📊 Starting input monitoring with real-time level meters');
    
    try {
        // Start backend monitoring stream
        const result = await invoke('start_input_monitoring');
        console.log('✅ Backend monitoring started:', result);
        
        isInputMonitoringEnabled = true;
        startLevelMeterUpdates();
        
    } catch (error) {
        console.error('❌ Failed to start backend monitoring:', error);
        // Reset UI on error
        const monitorBtn = document.getElementById('monitor-input-btn');
        monitorBtn.classList.remove('active');
        monitorBtn.textContent = '🎛️ Monitor Input';
        throw error;
    }
}

// Stop input monitoring
async function stopInputMonitoring() {
    console.log('📊 Stopping input monitoring');
    
    try {
        // Stop backend monitoring stream
        const result = await invoke('stop_input_monitoring');
        console.log('✅ Backend monitoring stopped:', result);
        
    } catch (error) {
        console.error('❌ Failed to stop backend monitoring:', error);
        // Continue with UI cleanup even if backend fails
    }
    
    isInputMonitoringEnabled = false;
    stopLevelMeterUpdates();
    
    // Reset meters to offline state
    updateLevelMeterDisplay(null);
}

// Internal function to start level meter UI updates
function startLevelMeterUpdates() {
    if (levelMeterUpdateInterval) {
        clearInterval(levelMeterUpdateInterval);
    }
    
    isLevelMeterActive = true;
    
    // Update loop only runs when monitoring is enabled
    levelMeterUpdateInterval = setInterval(async () => {
        if (!isLevelMeterActive || !isInputMonitoringEnabled) {
            return;
        }
        
        try {
            // Query backend for current audio levels
            const audioLevels = await invoke('get_audio_levels');
            updateLevelMeterDisplay(audioLevels);
            
        } catch (error) {
            // Show offline state on error
            updateLevelMeterDisplay(null);
        }
    }, LEVEL_METER_CONFIG.updateIntervalMs);
    
    console.log('✅ Level meter updates started');
}

// Internal function to stop level meter UI updates
function stopLevelMeterUpdates() {
    isLevelMeterActive = false;
    
    if (levelMeterUpdateInterval) {
        clearInterval(levelMeterUpdateInterval);
        levelMeterUpdateInterval = null;
    }
    
    console.log('✅ Level meter updates stopped');
}


// Update level meter UI components with audio data
function updateLevelMeterDisplay(audioLevels) {
    const meterFill = document.getElementById('input-meter-fill');
    const peakHold = document.getElementById('input-peak-hold');
    const levelReadout = document.getElementById('input-level-readout');
    const clippingWarning = document.getElementById('clipping-warning');
    const metersPanel = document.getElementById('level-meters-panel');
    
    if (!meterFill || !peakHold || !levelReadout) {
        return; // UI elements not available
    }
    
    if (!audioLevels) {
        // Show offline state
        metersPanel.classList.add('meters-offline');
        meterFill.style.width = '0%';
        peakHold.style.left = '0%';
        levelReadout.textContent = '-∞ dB';
        clippingWarning.style.display = 'none';
        return;
    }
    
    // Remove offline state
    metersPanel.classList.remove('meters-offline');
    
    const currentDb = audioLevels.rms_db;
    const peakDb = audioLevels.peak_db;
    
    // Convert dB to percentage for visual display (professional VU-style)
    // -60dB = 0%, 0dB = 100%
    const rmsPercent = dbToPercent(currentDb);
    const peakPercent = dbToPercent(peakDb);
    
    // Update RMS level bar (smooth VU-style movement)
    meterFill.style.width = `${rmsPercent}%`;
    
    // Professional peak hold logic with decay animation
    updatePeakHoldDisplay(peakDb, peakPercent, peakHold);
    
    // Update digital readout with precision
    if (currentDb <= LEVEL_METER_CONFIG.dbFloor) {
        levelReadout.textContent = '-∞ dB';
    } else {
        levelReadout.textContent = `${currentDb.toFixed(1)} dB`;
    }
    
    // Clipping detection and warning
    const isClipping = peakDb >= -0.1; // Near 0 dBFS
    clippingWarning.style.display = isClipping ? 'block' : 'none';
    
    // Professional color zones based on level
    updateMeterColors(meterFill, rmsPercent);
}

// Convert dB to percentage for meter display
function dbToPercent(db) {
    if (db <= LEVEL_METER_CONFIG.dbFloor) return 0;
    if (db >= LEVEL_METER_CONFIG.dbCeiling) return 100;
    
    // Linear conversion from dB range to 0-100%
    const range = LEVEL_METER_CONFIG.dbCeiling - LEVEL_METER_CONFIG.dbFloor;
    const normalized = (db - LEVEL_METER_CONFIG.dbFloor) / range;
    return Math.max(0, Math.min(100, normalized * 100));
}

// Professional peak hold with decay animation
function updatePeakHoldDisplay(currentPeakDb, currentPeakPercent, peakHoldElement) {
    const now = Date.now();
    
    // If current peak is higher, update peak hold
    if (currentPeakDb > inputPeakHold.level) {
        inputPeakHold.level = currentPeakDb;
        inputPeakHold.timestamp = now;
        inputPeakHold.isDecaying = false;
    }
    
    // Check if peak hold should start decaying
    if (!inputPeakHold.isDecaying && 
        (now - inputPeakHold.timestamp) > LEVEL_METER_CONFIG.peakHoldTimeMs) {
        inputPeakHold.isDecaying = true;
    }
    
    // Apply decay if active
    if (inputPeakHold.isDecaying) {
        inputPeakHold.level -= LEVEL_METER_CONFIG.peakDecayRate;
        
        // Don't decay below current level or floor
        if (inputPeakHold.level < Math.max(currentPeakDb, LEVEL_METER_CONFIG.dbFloor)) {
            inputPeakHold.level = Math.max(currentPeakDb, LEVEL_METER_CONFIG.dbFloor);
            inputPeakHold.isDecaying = false;
        }
    }
    
    // Update peak hold position
    const holdPercent = dbToPercent(inputPeakHold.level);
    peakHoldElement.style.left = `${holdPercent}%`;
}

// Professional meter color zones (broadcast standard)
function updateMeterColors(meterFill, percent) {
    // The CSS gradient handles colors automatically based on percentage:
    // Green: 0-67% (-60 to -20dB)
    // Yellow: 67-85% (-20 to -9dB)  
    // Red: 85-100% (-9 to 0dB)
    
    // Colors are handled by CSS gradient, no JavaScript needed
    // This function is reserved for future advanced color features
}


// Enhanced recording functions with level meter integration
function startRecordingWithMeters() {
    console.log('🔴 Starting recording with active level monitoring');
    
    // Ensure level meters are running during recording
    if (!isLevelMeterActive) {
        startLevelMeters();
    }
    
    // TODO: Update meter state to show recording (red indicator)
    const metersPanel = document.getElementById('level-meters-panel');
    if (metersPanel) {
        metersPanel.classList.add('recording-active');
    }
}

function stopRecordingWithMeters() {
    console.log('⏹️ Stopping recording, maintaining level monitoring');
    
    // Remove recording state from meters
    const metersPanel = document.getElementById('level-meters-panel');
    if (metersPanel) {
        metersPanel.classList.remove('recording-active');
    }
    
    // Keep level meters running for continued monitoring
}

// Initialize level meter system when page loads
document.addEventListener('DOMContentLoaded', () => {
    console.log('📊 Initializing AKAI-style level meter system');
    
    // Initialize meters in offline state (user must click Monitor Input to activate)
    updateLevelMeterDisplay(null);
});

// Export level meter functions to global scope
window.toggleInputMonitoring = toggleInputMonitoring;
window.startInputMonitoring = startInputMonitoring;
window.stopInputMonitoring = stopInputMonitoring;
window.updateLevelMeterDisplay = updateLevelMeterDisplay;

// ============================================================================
// PROJECT TEMPLATES SYSTEM
// ============================================================================

let templatesVisible = false;

// Toggle templates panel visibility
function toggleTemplatesPanel() {
    const templatesContent = document.getElementById('templates-content');
    const toggleBtn = document.getElementById('templates-toggle-btn');
    
    if (templatesVisible) {
        templatesContent.style.display = 'none';
        toggleBtn.textContent = 'Show Templates';
        templatesVisible = false;
    } else {
        populateTemplatesGrid(); // Populate on first show
        templatesContent.style.display = 'block';
        toggleBtn.textContent = 'Hide Templates';
        templatesVisible = true;
    }
}

// Populate the templates dropdown with available templates  
function populateTemplatesDropdown() {
    const templateDropdown = document.getElementById('template-dropdown');
    if (!templateDropdown) return;
    
    // Get all templates
    const templates = getAllTemplates();
    
    // Clear existing options except the first one
    while (templateDropdown.children.length > 1) {
        templateDropdown.removeChild(templateDropdown.lastChild);
    }
    
    templates.forEach(template => {
        const option = document.createElement('option');
        option.value = template.id;
        option.textContent = template.name;
        templateDropdown.appendChild(option);
    });
}

// Apply selected template from dropdown
function applySelectedTemplate() {
    const templateDropdown = document.getElementById('template-dropdown');
    if (!templateDropdown || !templateDropdown.value) return;
    
    console.log('🎯 Applying template:', templateDropdown.value);
    applyTemplateById(templateDropdown.value);
}

// Populate the templates grid with available templates (legacy for any remaining uses)
function populateTemplatesGrid() {
    const templatesGrid = document.getElementById('templates-grid');
    if (!templatesGrid) return;
    
    // Clear existing content
    templatesGrid.innerHTML = '';
    
    // Get all templates
    const templates = getAllTemplates();
    
    templates.forEach(template => {
        const templateCard = createTemplateCard(template);
        templatesGrid.appendChild(templateCard);
    });
}

// Create a template card element
function createTemplateCard(template) {
    const card = document.createElement('div');
    card.className = 'template-card';
    card.setAttribute('data-template-id', template.id);
    
    card.innerHTML = `
        <div class="template-header">
            <div class="template-icon">${template.icon}</div>
            <h4 class="template-name">${template.name}</h4>
        </div>
        <div class="template-description">${template.description}</div>
        <div class="template-stats">
            <span class="template-samples">${template.estimatedSamples} samples</span>
            <span class="template-time">${template.estimatedTime}</span>
        </div>
        <button class="template-apply-btn" onclick="applyTemplateById('${template.id}')">Apply</button>
    `;
    
    // Add click handler for the whole card
    card.addEventListener('click', (e) => {
        // Don't trigger card click if apply button was clicked
        if (e.target.classList.contains('template-apply-btn')) return;
        
        // Show template details or apply directly
        applyTemplateById(template.id);
    });
    
    return card;
}

// Apply a template by ID
function applyTemplateById(templateId) {
    console.log('🎯 Applying template:', templateId);
    
    const success = applyTemplate(templateId);
    
    if (success) {
        // Update visual feedback
        updateTemplateSelection(templateId);
        
        // Optionally hide templates panel after selection
        // toggleTemplatesPanel();
    }
}

// Update visual selection state
function updateTemplateSelection(selectedTemplateId) {
    const templateCards = document.querySelectorAll('.template-card');
    
    templateCards.forEach(card => {
        const templateId = card.getAttribute('data-template-id');
        if (templateId === selectedTemplateId) {
            card.classList.add('selected');
        } else {
            card.classList.remove('selected');
        }
    });
}

// Initialize templates on page load
document.addEventListener('DOMContentLoaded', () => {
    console.log('🎯 Initializing project templates system');
    
    // Templates will be populated when panel is first shown
    // This avoids DOM manipulation during initial page load
});

// Export template functions to global scope
window.toggleTemplatesPanel = toggleTemplatesPanel;
window.populateTemplatesGrid = populateTemplatesGrid;
window.populateTemplatesDropdown = populateTemplatesDropdown;
window.applySelectedTemplate = applySelectedTemplate;
window.applyTemplateById = applyTemplateById;
window.updateTemplateSelection = updateTemplateSelection;


// Debug: Verify functions are available
console.log('🔧 Functions exported to window:', {
    loadMidiDevices: typeof window.loadMidiDevices,
    testMidiConnection: typeof window.testMidiConnection,
    previewNote: typeof window.previewNote,
    recordSample: typeof window.recordSample,
    showWaveform: typeof window.showWaveform
});

// ============================================================================
// LOOP DETECTION SYSTEM
// ============================================================================

// Global audio element for loop preview
let loopPreviewAudio = null;
let isLoopPlaying = false;

// Preview the detected loop point by playing just that segment
async function previewLoopPoint() {
    console.log('🚀 PREVIEW BUTTON CLICKED!');
    
    if (!window.currentLoopData) {
        console.error('❌ No currentLoopData');
        showStatus('No loop data available', 'error');
        return;
    }
    
    if (!window.currentLoopData.filePath) {
        console.error('❌ No filePath in currentLoopData:', window.currentLoopData);
        showStatus('No file path available for preview', 'error');
        return;
    }
    
    const loopData = window.currentLoopData;
    console.log('🎵 Loop data:', {
        filePath: loopData.filePath,
        startTime: loopData.startTime,
        endTime: loopData.endTime,
        length: loopData.endTime - loopData.startTime
    });
    
    try {
        const previewBtn = document.getElementById('preview-loop-btn');
        console.log('🔘 Preview button found:', !!previewBtn);
        
        if (isLoopPlaying) {
            console.log('⏹️ Stopping current playback');
            // Stop current playback
            if (loopPreviewAudio) {
                loopPreviewAudio.pause();
                loopPreviewAudio.currentTime = 0;
            }
            isLoopPlaying = false;
            if (previewBtn) {
                previewBtn.textContent = '🎵 Preview Loop';
                previewBtn.classList.remove('btn-danger');
                previewBtn.classList.add('btn-primary');
            }
            showStatus('Preview stopped', 'info');
            return;
        }
        
        // Convert file path for Tauri
        const audioFileUrl = convertFileSrc(loopData.filePath);
        console.log('🔧 Converted audio URL:', audioFileUrl);
        
        // Create or reuse audio element
        if (!loopPreviewAudio) {
            loopPreviewAudio = new Audio();
            console.log('🎵 Created new Audio element');
        }
        
        loopPreviewAudio.src = audioFileUrl;
        console.log('📁 Set audio source');
        
        // Update button to show loading
        if (previewBtn) {
            previewBtn.textContent = '⏳ Loading...';
        }
        showStatus('Loading audio for preview...', 'info');
        
        // Wait for audio to load with timeout
        console.log('⏳ Waiting for audio to load...');
        await new Promise((resolve, reject) => {
            const timeout = setTimeout(() => {
                reject(new Error('Audio loading timeout'));
            }, 10000);
            
            loopPreviewAudio.onloadeddata = () => {
                console.log('✅ Audio loaded successfully');
                console.log(`📊 Audio duration: ${loopPreviewAudio.duration}s`);
                clearTimeout(timeout);
                resolve();
            };
            
            loopPreviewAudio.onerror = (e) => {
                console.error('❌ Audio loading error:', e);
                clearTimeout(timeout);
                reject(new Error('Failed to load audio file'));
            };
        });
        
        // Validate loop times against audio duration
        if (loopData.startTime >= loopPreviewAudio.duration) {
            throw new Error(`Start time ${loopData.startTime}s exceeds audio duration ${loopPreviewAudio.duration}s`);
        }
        if (loopData.endTime > loopPreviewAudio.duration) {
            console.warn(`⚠️ End time ${loopData.endTime}s exceeds duration ${loopPreviewAudio.duration}s, clamping`);
            loopData.endTime = loopPreviewAudio.duration;
        }
        
        // Set up loop playback
        loopPreviewAudio.currentTime = loopData.startTime;
        console.log(`🎯 Set playback position to ${loopData.startTime}s`);
        
        // Update button to show playing state
        if (previewBtn) {
            previewBtn.textContent = '⏹️ Stop Preview';
            previewBtn.classList.remove('btn-primary');
            previewBtn.classList.add('btn-danger');
        }
        
        isLoopPlaying = true;
        
        // Play the audio
        console.log('▶️ Starting playback...');
        await loopPreviewAudio.play();
        console.log('✅ Playback started');
        
        showStatus(`Playing loop: ${loopData.startTime.toFixed(2)}s - ${loopData.endTime.toFixed(2)}s`, 'success');
        
        // Monitor playback and loop
        const checkPosition = () => {
            if (!isLoopPlaying || loopPreviewAudio.paused) {
                console.log('🛑 Playback monitoring stopped');
                return;
            }
            
            const currentTime = loopPreviewAudio.currentTime;
            
            if (currentTime >= loopData.endTime) {
                console.log(`🔄 Looping: ${currentTime.toFixed(3)}s >= ${loopData.endTime.toFixed(3)}s, jumping to ${loopData.startTime.toFixed(3)}s`);
                loopPreviewAudio.currentTime = loopData.startTime;
            }
            
            requestAnimationFrame(checkPosition);
        };
        
        checkPosition();
        
        // Auto-stop after 10 seconds
        setTimeout(() => {
            if (isLoopPlaying) {
                previewLoopPoint(); // This will stop the playback
            }
        }, 10000);
        
        console.log('✅ Loop preview started');
        
    } catch (error) {
        console.error('❌ Failed to preview loop:', error);
        showStatus(`Failed to preview loop: ${error.message}`, 'error');
        
        // Reset button state
        const previewBtn = document.getElementById('preview-loop-btn');
        if (previewBtn) {
            previewBtn.textContent = '🎵 Preview Loop';
            previewBtn.classList.remove('btn-danger');
            previewBtn.classList.add('btn-primary');
        }
        isLoopPlaying = false;
    }
}

// Accept the loop point and apply it to the sample
async function acceptLoopPoint() {
    if (!window.currentLoopData || !window.currentLoopData.filePath) {
        console.error('❌ No loop data available to apply');
        showStatus('No loop data available to apply', 'error');
        return;
    }
    
    const loopData = window.currentLoopData;
    console.log('✅ Accepting loop point:', loopData);
    
    try {
        // Call backend to apply loop metadata to the file
        const result = await invoke('apply_loop_metadata', {
            filePath: loopData.filePath,
            startSample: loopData.startSample,
            endSample: loopData.endSample,
            sampleRate: loopData.sampleRate
        });
        
        console.log('✅ Loop metadata applied:', result);
        showStatus('Loop points applied successfully! Future exports will include loop data.', 'success');
        
        // Update UI to show applied state
        const acceptBtn = document.getElementById('accept-loop-btn');
        if (acceptBtn) {
            acceptBtn.textContent = '✅ Applied';
            acceptBtn.disabled = true;
            acceptBtn.classList.remove('btn-success');
            acceptBtn.classList.add('btn-secondary');
        }
        
    } catch (error) {
        console.error('❌ Failed to apply loop metadata:', error);
        showStatus(`Failed to apply loop: ${error}`, 'error');
    }
}

// Show alternative loop candidates (if multiple were found)
function showAlternativeLoops() {
    console.log('📋 Showing alternative loop candidates...');
    
    if (!window.currentLoopDetectionResult || !window.currentLoopDetectionResult.candidates) {
        console.warn('⚠️ No loop candidates data available');
        showStatus('No loop candidates data available', 'error');
        return;
    }
    
    const candidates = window.currentLoopDetectionResult.candidates;
    const sampleRate = window.currentLoopDetectionResult.sample_rate || 44100;
    
    if (candidates.length <= 1) {
        showStatus('Only one loop candidate found', 'info');
        return;
    }
    
    // Create modal overlay for loop candidates selection
    createLoopCandidatesModal(candidates, sampleRate);
}

// Create professional loop candidates selection modal (SampleRobot style)
function createLoopCandidatesModal(candidates, sampleRate) {
    // Remove existing modal if present
    const existingModal = document.getElementById('loop-candidates-modal');
    if (existingModal) {
        existingModal.remove();
    }
    
    // Create modal HTML
    const modal = document.createElement('div');
    modal.id = 'loop-candidates-modal';
    modal.className = 'modal-overlay';
    modal.style.cssText = `
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.8);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
        backdrop-filter: blur(4px);
    `;
    
    // Create modal content
    const modalContent = document.createElement('div');
    modalContent.className = 'modal-content';
    modalContent.style.cssText = `
        background: var(--bg-elevated);
        border-radius: 8px;
        padding: 24px;
        width: 90%;
        max-width: 800px;
        max-height: 80vh;
        overflow-y: auto;
        box-shadow: var(--shadow-heavy);
        border: 1px solid var(--border-primary);
    `;
    
    // Generate candidates list HTML
    const candidatesHTML = candidates.map((candidate, index) => {
        const startTime = (candidate.start_sample / sampleRate).toFixed(3);
        const endTime = (candidate.end_sample / sampleRate).toFixed(3);
        const lengthTime = (candidate.length_samples / sampleRate).toFixed(3);
        const quality = Math.round(candidate.quality_score * 100);
        const isSelected = index === 0; // First candidate is currently selected
        const defaultBorderColor = isSelected ? 'var(--accent-primary)' : 'var(--border-secondary)';
        
        return `
            <div class="loop-candidate ${isSelected ? 'selected' : ''}" 
                 data-candidate-index="${index}"
                 style="
                     background: ${isSelected ? 'rgba(70, 130, 180, 0.2)' : 'var(--bg-secondary)'};
                     border: 2px solid ${isSelected ? 'var(--accent-primary)' : 'var(--border-secondary)'};
                     border-radius: 6px;
                     padding: 16px;
                     margin-bottom: 12px;
                     cursor: pointer;
                     transition: all 0.2s ease;
                 "
                 onclick="selectLoopCandidate(${index})"
                 onmouseover="this.style.borderColor='var(--accent-primary)'"
                 onmouseout="this.style.borderColor='${defaultBorderColor}'">
                
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                    <div style="font-weight: 600; color: var(--text-primary);">
                        ${isSelected ? '🏆 ' : ''}Loop Candidate ${index + 1}
                        ${isSelected ? ' (Current)' : ''}
                    </div>
                    <div style="
                        padding: 4px 12px;
                        border-radius: 12px;
                        background: ${quality >= 70 ? 'rgba(34, 197, 94, 0.2)' : 
                                   quality >= 50 ? 'rgba(245, 158, 11, 0.2)' : 
                                   'rgba(239, 68, 68, 0.2)'};
                        color: ${quality >= 70 ? '#22c55e' : 
                               quality >= 50 ? '#f59e0b' : 
                               '#ef4444'};
                        font-size: 12px;
                        font-weight: 600;
                    ">
                        ${quality}% Quality
                    </div>
                </div>
                
                <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; font-size: 13px;">
                    <div>
                        <div style="color: var(--text-secondary); font-size: 11px; text-transform: uppercase;">Start</div>
                        <div style="color: var(--text-primary); font-weight: 500;">${startTime}s</div>
                    </div>
                    <div>
                        <div style="color: var(--text-secondary); font-size: 11px; text-transform: uppercase;">End</div>
                        <div style="color: var(--text-primary); font-weight: 500;">${endTime}s</div>
                    </div>
                    <div>
                        <div style="color: var(--text-secondary); font-size: 11px; text-transform: uppercase;">Length</div>
                        <div style="color: var(--text-primary); font-weight: 500;">${lengthTime}s</div>
                    </div>
                    <div>
                        <div style="color: var(--text-secondary); font-size: 11px; text-transform: uppercase;">Zero-Cross</div>
                        <div style="color: var(--text-primary); font-weight: 500;">
                            ${candidate.zero_crossing_aligned ? '✓ Yes' : '✗ No'}
                        </div>
                    </div>
                </div>
                
                <div style="margin-top: 12px; display: flex; gap: 8px;">
                    <button class="btn btn-sm btn-primary" 
                            onclick="previewLoopCandidate(${index}); event.stopPropagation();"
                            style="padding: 4px 12px; font-size: 12px;">
                        ▶️ Preview
                    </button>
                    <button class="btn btn-sm btn-success" 
                            onclick="selectAndApplyLoopCandidate(${index}); event.stopPropagation();"
                            style="padding: 4px 12px; font-size: 12px;">
                        ✓ Select & Apply
                    </button>
                </div>
            </div>
        `;
    }).join('');
    
    modalContent.innerHTML = `
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
            <h2 style="margin: 0; color: var(--text-primary); font-size: 20px;">
                Loop Candidates Selection
            </h2>
            <button class="btn btn-sm btn-secondary" onclick="closeLoopCandidatesModal()" 
                    style="padding: 8px 16px;">
                ✕ Close
            </button>
        </div>
        
        <div style="margin-bottom: 16px; padding: 12px; background: rgba(70, 130, 180, 0.1); 
                    border-radius: 6px; border-left: 4px solid var(--accent-primary);">
            <div style="font-size: 14px; color: var(--text-primary); margin-bottom: 4px;">
                Found ${candidates.length} loop candidates, ranked by quality
            </div>
            <div style="font-size: 12px; color: var(--text-secondary);">
                Click a candidate to preview it on the waveform, or select "Select & Apply" to use it.
            </div>
        </div>
        
        <div id="loop-candidates-list">
            ${candidatesHTML}
        </div>
    `;
    
    modal.appendChild(modalContent);
    document.body.appendChild(modal);
    
    // Close modal when clicking outside
    modal.addEventListener('click', (e) => {
        if (e.target === modal) {
            closeLoopCandidatesModal();
        }
    });
    
    console.log('🎯 Created loop candidates modal with', candidates.length, 'candidates');
}

// Modal interaction functions
function closeLoopCandidatesModal() {
    const modal = document.getElementById('loop-candidates-modal');
    if (modal) {
        modal.remove();
        console.log('🗑️ Closed loop candidates modal');
    }
}

function selectLoopCandidate(candidateIndex) {
    if (!window.currentLoopDetectionResult || !window.currentLoopDetectionResult.candidates) {
        console.error('❌ No candidates data available');
        return;
    }
    
    const candidate = window.currentLoopDetectionResult.candidates[candidateIndex];
    const sampleRate = window.currentLoopDetectionResult.sample_rate || 44100;
    
    if (!candidate) {
        console.error('❌ Invalid candidate index:', candidateIndex);
        return;
    }
    
    console.log(`🎯 Selected loop candidate ${candidateIndex + 1}:`, candidate);
    
    // Update current loop data
    window.currentLoopData = {
        filePath: window.currentLoopData?.filePath || null,
        startSample: candidate.start_sample,
        endSample: candidate.end_sample,
        startTime: candidate.start_sample / sampleRate,
        endTime: candidate.end_sample / sampleRate,
        sampleRate: sampleRate,
        qualityScore: candidate.quality_score
    };
    
    // Update visual markers on waveform
    const wavesurfer = wavesurferInstance || rangeWavesurferInstance;
    if (wavesurfer && wavesurfer.regionsPlugin) {
        addLoopMarkersToWaveform(
            wavesurfer,
            window.currentLoopData.startTime,
            window.currentLoopData.endTime,
            true
        );
    }
    
    // Update UI selection state
    document.querySelectorAll('.loop-candidate').forEach((element, index) => {
        const isSelected = index === candidateIndex;
        element.className = `loop-candidate ${isSelected ? 'selected' : ''}`;
        element.style.background = isSelected ? 'rgba(70, 130, 180, 0.2)' : 'var(--bg-secondary)';
        element.style.borderColor = isSelected ? 'var(--accent-primary)' : 'var(--border-secondary)';
        
        // Update title text
        const title = element.querySelector('div > div');
        if (title) {
            title.innerHTML = `${isSelected ? '🏆 ' : ''}Loop Candidate ${index + 1}${isSelected ? ' (Current)' : ''}`;
        }
    });
    
    showStatus(`Selected loop candidate ${candidateIndex + 1} (${Math.round(candidate.quality_score * 100)}% quality)`, 'success');
}

function previewLoopCandidate(candidateIndex) {
    console.log('🚀 previewLoopCandidate called with index:', candidateIndex);
    
    if (!window.currentLoopDetectionResult || !window.currentLoopDetectionResult.candidates) {
        console.error('❌ No candidates data available');
        return;
    }
    
    const candidate = window.currentLoopDetectionResult.candidates[candidateIndex];
    const sampleRate = window.currentLoopDetectionResult.sample_rate || 44100;
    
    if (!candidate) {
        console.error('❌ Invalid candidate index:', candidateIndex);
        return;
    }
    
    console.log(`🎵 Previewing loop candidate ${candidateIndex + 1}:`, {
        startSample: candidate.start_sample,
        endSample: candidate.end_sample,
        startTime: (candidate.start_sample / sampleRate).toFixed(3),
        endTime: (candidate.end_sample / sampleRate).toFixed(3),
        quality: Math.round(candidate.quality_score * 100)
    });
    
    // Temporarily update loop data for preview
    const originalLoopData = { ...window.currentLoopData };
    window.currentLoopData = {
        filePath: window.currentLoopData?.filePath || null,
        startSample: candidate.start_sample,
        endSample: candidate.end_sample,
        startTime: candidate.start_sample / sampleRate,
        endTime: candidate.end_sample / sampleRate,
        sampleRate: sampleRate,
        qualityScore: candidate.quality_score
    };
    
    // Preview this candidate
    previewLoopPoint().then(() => {
        // Show temporary visual feedback
        showStatus(`Previewing candidate ${candidateIndex + 1} - ${Math.round(candidate.quality_score * 100)}% quality`, 'info');
        
        // Restore original loop data after preview
        setTimeout(() => {
            window.currentLoopData = originalLoopData;
        }, 100);
    }).catch((error) => {
        console.error('❌ Preview failed:', error);
        window.currentLoopData = originalLoopData;
    });
}

function selectAndApplyLoopCandidate(candidateIndex) {
    // First select the candidate
    selectLoopCandidate(candidateIndex);
    
    // Then apply it
    acceptLoopPoint().then(() => {
        // Close the modal after successful application
        closeLoopCandidatesModal();
        showStatus(`Applied loop candidate ${candidateIndex + 1} successfully!`, 'success');
    }).catch((error) => {
        console.error('❌ Failed to apply candidate:', error);
        showStatus(`Failed to apply candidate: ${error}`, 'error');
    });
}

// Test loop detection on the last recorded sample
async function testLoopDetection() {
    console.log('🔄 Testing loop detection on last recorded sample...');
    
    // Try both element naming conventions (index.html vs temp_body.html)
    const statusElement = document.getElementById('loop-detection-status') || document.getElementById('loop-detection-result');
    const resultsElement = document.getElementById('loop-detection-results') || document.getElementById('loop-detection-result');
    
    if (statusElement) {
        statusElement.textContent = 'Running loop detection...';
        statusElement.style.display = 'block';
    }
    
    try {
        // Get the last recorded sample file from the system
        const lastSamplePath = await getLastRecordedSamplePath();
        
        if (!lastSamplePath) {
            throw new Error('No recorded sample found. Please record a sample first.');
        }
        
        console.log('🎵 Testing loop detection on:', lastSamplePath);
        
        // Get loop detection parameters from UI
        const params = getLoopDetectionParams();
        console.log('🔧 Loop detection parameters:', params);
        
        // Call backend loop detection
        const result = await invoke('detect_loop_points', {
            filePath: lastSamplePath,
            minLoopLength: params.minLoopLength,
            maxLoopLength: params.maxLoopLength,
            correlationThreshold: params.correlationThreshold
        });
        
        console.log('✅ Loop detection result:', result);
        
        // Display results (this will create window.currentLoopData)
        displayLoopDetectionResults(result, statusElement, resultsElement);
        
        // Store the file path for loop preview/apply functionality
        if (window.currentLoopData) {
            window.currentLoopData.filePath = lastSamplePath;
            console.log('📂 Set file path in loop data:', lastSamplePath);
        }
        
    } catch (error) {
        console.error('❌ Loop detection failed:', error);
        
        if (statusElement) {
            statusElement.textContent = `Loop detection failed: ${error}`;
            statusElement.style.color = '#dc2626';
        }
        
        if (resultsElement) {
            resultsElement.innerHTML = `<div style="color: #dc2626; padding: 10px;">Error: ${error}</div>`;
        }
        
        showStatus(`Loop detection failed: ${error}`, 'error');
    }
}

// Get loop detection parameters from UI sliders
function getLoopDetectionParams() {
    // Try both naming conventions (index.html vs temp_body.html)
    const minLoopSlider = document.getElementById('min-loop-length') || document.getElementById('loop-min-length');
    const maxLoopSlider = document.getElementById('max-loop-length') || document.getElementById('loop-max-length');
    const correlationSlider = document.getElementById('correlation-threshold') || document.getElementById('loop-correlation-threshold');
    
    const params = {
        minLoopLength: minLoopSlider ? parseFloat(minLoopSlider.value) : 0.1,
        maxLoopLength: maxLoopSlider ? parseFloat(maxLoopSlider.value) : 5.0,
        correlationThreshold: correlationSlider ? parseFloat(correlationSlider.value) : 0.5  // Lower default
    };
    
    console.log('🔧 Loop detection params from UI:', params);
    console.log('🔧 UI elements found:', {
        minSlider: !!minLoopSlider,
        maxSlider: !!maxLoopSlider, 
        correlationSlider: !!correlationSlider
    });
    
    return params;
}

// Get the path of the last recorded sample
async function getLastRecordedSamplePath() {
    try {
        // Try to get from the most recent recording
        const outputDir = document.getElementById('output-directory')?.value;
        const sampleName = document.getElementById('sample-name')?.value?.trim();
        
        // Use backend to find the last recorded file
        const result = await invoke('get_last_recorded_sample_path', {
            outputDirectory: outputDir || null,
            sampleName: sampleName || null
        });
        
        return result;
        
    } catch (error) {
        console.error('❌ Failed to get last sample path:', error);
        return null;
    }
}

// Display loop detection results in the UI
function displayLoopDetectionResults(result, statusElement, resultsElement) {
    try {
        const loopResult = JSON.parse(result);
        
        // Store globally for loop candidates modal
        window.currentLoopDetectionResult = loopResult;
        
        // Find the correct status element if not provided or null
        const actualStatusElement = statusElement || 
            document.getElementById('loop-detection-status') || 
            document.getElementById('loop-detection-result');
            
        // Find the correct results element if not provided or null  
        const actualResultsElement = resultsElement || 
            document.getElementById('loop-detection-results') || 
            document.getElementById('loop-detection-result');
        
        if (actualStatusElement) {
            if (loopResult.success) {
                actualStatusElement.textContent = 'Loop detection completed successfully!';
                actualStatusElement.style.color = '#16a34a';
            } else {
                actualStatusElement.textContent = `Loop detection failed: ${loopResult.failure_reason || 'Unknown error'}`;
                actualStatusElement.style.color = '#dc2626';
            }
        } else {
            console.warn('⚠️ No status element found for loop detection results');
        }
        
        if (actualResultsElement) {
            if (loopResult.success && loopResult.best_candidate) {
                const sampleRate = loopResult.sample_rate || 44100;
                const candidate = loopResult.best_candidate;
                const startSec = (candidate.start_sample / sampleRate).toFixed(3);
                const endSec = (candidate.end_sample / sampleRate).toFixed(3);
                const lengthSec = (candidate.length_samples / sampleRate).toFixed(3);
                const qualityPercent = Math.round(candidate.quality_score * 100);
                
                // Store loop data globally for preview and apply functions
                window.currentLoopData = {
                    filePath: null, // Will be set when we get the file path
                    startSample: candidate.start_sample,
                    endSample: candidate.end_sample,
                    startTime: parseFloat(startSec),
                    endTime: parseFloat(endSec),
                    sampleRate: sampleRate,
                    qualityScore: candidate.quality_score
                };
                
                const html = `
                    <div style="background: rgba(34, 197, 94, 0.1); border: 1px solid rgba(34, 197, 94, 0.3); border-radius: 6px; padding: 12px; margin-top: 8px;">
                        <div style="font-weight: 600; color: #16a34a; margin-bottom: 8px;">✅ Loop Point Found</div>
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; font-size: 12px; margin-bottom: 12px;">
                            <div><strong>Start:</strong> ${startSec}s</div>
                            <div><strong>End:</strong> ${endSec}s</div>
                            <div><strong>Length:</strong> ${lengthSec}s</div>
                            <div><strong>Quality:</strong> ${qualityPercent}%</div>
                        </div>
                        <div style="display: flex; gap: 8px; margin-bottom: 8px;">
                            <button class="btn btn-sm btn-primary" onclick="previewLoopPoint()" id="preview-loop-btn">
                                🎵 Preview Loop
                            </button>
                            <button class="btn btn-sm btn-success" onclick="acceptLoopPoint()" id="accept-loop-btn">
                                ✅ Accept & Apply
                            </button>
                            ${loopResult.candidates.length > 1 ? 
                                `<button class="btn btn-sm btn-secondary" onclick="showAlternativeLoops()" id="alternatives-btn">
                                    📋 ${loopResult.candidates.length} Options
                                </button>` : 
                                ''
                            }
                        </div>
                        ${loopResult.candidates.length > 1 ? 
                            `<div style="font-size: 11px; color: #6b7280;">Found ${loopResult.candidates.length} candidates, showing best quality</div>` : 
                            ''
                        }
                    </div>
                `;
                actualResultsElement.innerHTML = html;
                
                // ALWAYS add visual loop markers to the waveform (SampleRobot style)
                const wavesurfer = wavesurferInstance || rangeWavesurferInstance;
                if (wavesurfer && wavesurfer.regionsPlugin) {
                    addLoopMarkersToWaveform(
                        wavesurfer,
                        window.currentLoopData.startTime,
                        window.currentLoopData.endTime,
                        true // Make it editable
                    );
                    console.log('🎯 Added visual loop markers to waveform');
                } else {
                    console.warn('⚠️ Cannot add visual markers: waveform not available');
                }
                
            } else {
                actualResultsElement.innerHTML = `
                    <div style="background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.3); border-radius: 6px; padding: 12px; margin-top: 8px;">
                        <div style="font-weight: 600; color: #f59e0b; margin-bottom: 8px;">⚠️ No Quality Loops Found</div>
                        <div style="font-size: 12px; margin-bottom: 8px;">This sample may not be suitable for looping, or the quality threshold is too strict.</div>
                        <div style="font-size: 11px; color: #6b7280;">
                            <strong>Try:</strong> Lower the Quality Threshold to 50-60% • Use longer sustained sounds • Record pads or sustained tones
                        </div>
                    </div>
                `;
            }
        }
        
        // ALWAYS show success - we never fail now
        showStatus(`Loop detection completed! Found ${loopResult.candidates.length} candidates.`, 'success');
        
    } catch (parseError) {
        console.error('❌ Failed to parse loop detection result:', parseError);
        console.error('❌ Raw result that failed to parse:', result);
        
        // Find the correct elements for error display
        const actualStatusElement = statusElement || 
            document.getElementById('loop-detection-status') || 
            document.getElementById('loop-detection-result');
            
        const actualResultsElement = resultsElement || 
            document.getElementById('loop-detection-results') || 
            document.getElementById('loop-detection-result');
        
        if (actualStatusElement) {
            actualStatusElement.textContent = 'Failed to parse loop detection result';
            actualStatusElement.style.color = '#dc2626';
        }
        
        if (actualResultsElement && actualResultsElement !== actualStatusElement) {
            actualResultsElement.innerHTML = `<div style="color: #dc2626; padding: 10px;">Parse error: ${parseError}</div>`;
        }
    }
}

// Setup loop detection slider displays
function setupLoopDetectionSliders() {
    // Min loop length slider
    const minLoopSlider = document.getElementById('min-loop-length');
    const minLoopDisplay = document.getElementById('min-loop-display');
    if (minLoopSlider && minLoopDisplay) {
        minLoopSlider.addEventListener('input', () => {
            minLoopDisplay.textContent = `${minLoopSlider.value}s`;
        });
    }
    
    // Max loop length slider
    const maxLoopSlider = document.getElementById('max-loop-length');
    const maxLoopDisplay = document.getElementById('max-loop-display');
    if (maxLoopSlider && maxLoopDisplay) {
        maxLoopSlider.addEventListener('input', () => {
            maxLoopDisplay.textContent = `${maxLoopSlider.value}s`;
        });
    }
    
    // Correlation threshold slider
    const correlationSlider = document.getElementById('correlation-threshold');
    const correlationDisplay = document.getElementById('correlation-display');
    if (correlationSlider && correlationDisplay) {
        correlationSlider.addEventListener('input', () => {
            correlationDisplay.textContent = (parseFloat(correlationSlider.value) * 100).toFixed(0) + '%';
        });
    }
}

// Initialize loop detection when page loads
document.addEventListener('DOMContentLoaded', () => {
    console.log('🔄 Initializing loop detection system');
    setupLoopDetectionSliders();
});

// Export loop detection functions to global scope
window.testLoopDetection = testLoopDetection;
window.getLoopDetectionParams = getLoopDetectionParams;
window.displayLoopDetectionResults = displayLoopDetectionResults;
window.previewLoopPoint = previewLoopPoint;
window.acceptLoopPoint = acceptLoopPoint;
window.showAlternativeLoops = showAlternativeLoops;
window.closeLoopCandidatesModal = closeLoopCandidatesModal;
window.selectLoopCandidate = selectLoopCandidate;
window.previewLoopCandidate = previewLoopCandidate;
window.selectAndApplyLoopCandidate = selectAndApplyLoopCandidate;

// ============================================================================
// COLLAPSIBLE SECTIONS SYSTEM
// ============================================================================

function toggleAdvancedSettings(sectionId) {
    const content = document.getElementById(`${sectionId}-content`);
    const indicator = document.getElementById(`${sectionId}-indicator`);
    const header = indicator?.parentElement;
    
    if (!content || !indicator) {
        console.error(`❌ Could not find collapsible elements for: ${sectionId}`);
        return;
    }
    
    const isCollapsed = content.classList.contains('collapsed');
    
    if (isCollapsed) {
        // Expand
        content.classList.remove('collapsed');
        indicator.textContent = '▼';
        header?.classList.remove('collapsed');
    } else {
        // Collapse
        content.classList.add('collapsed');
        indicator.textContent = '▶';
        header?.classList.add('collapsed');
    }
}

// Handle keyboard navigation for collapsible sections
function handleKeyboardToggle(event, sectionId) {
    if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        toggleAdvancedSettings(sectionId);
    }
}

// Export collapsible functions to global scope
window.toggleAdvancedSettings = toggleAdvancedSettings;
window.handleKeyboardToggle = handleKeyboardToggle;