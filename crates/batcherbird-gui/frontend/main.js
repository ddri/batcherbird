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
        recordBtn.textContent = '⏹️ Recording...';
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
        
        try {
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
            
            // Extract file path from result message and show waveform
            setTimeout(async () => {
                try {
                    // Parse the file path from the result message
                    // Example result: "Recording saved: DW6000_C4_60_vel127.wav (45056 samples)\nLocation: /path/to/file.wav"
                    const locationMatch = result.match(/Location: (.+\.wav)/);
                    if (locationMatch) {
                        const filePath = locationMatch[1];
                        console.log('🌊 Showing waveform for recorded file:', filePath);
                        await showWaveform(filePath, false);
                    } else {
                        console.log('ℹ️ Could not extract file path for waveform display');
                    }
                } catch (waveformError) {
                    console.error('❌ Failed to show waveform:', waveformError);
                }
            }, 1000); // Delay to let file system sync
            
        } catch (backendError) {
            console.error('❌ Backend recording failed:', backendError);
            recordingText.textContent = 'Recording failed!';
            showStatus(`Recording failed: ${backendError}`, 'error');
            throw backendError; // Re-throw to be caught by outer try-catch
        }
        
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
        recordBtn.textContent = '🔴 Record Sample';
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
    rangeCurrentNote.textContent = '⏹️ Recording cancelled by user';
    
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
                        console.log('🌊 Showing range waveform for:', filePath);
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
                devices.forEach((device, index) => {
                    const option = document.createElement('option');
                    option.value = index.toString();
                    option.textContent = device;
                    midiSelect.appendChild(option);
                    
                    // Auto-select if this was the previously selected device
                    if (device === selectedMidiDevice) {
                        option.selected = true;
                        connectMidiDevice(index);
                    }
                });
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
                devices.forEach((device, index) => {
                    const option = document.createElement('option');
                    option.value = index.toString();
                    option.textContent = device;
                    audioInputSelect.appendChild(option);
                    
                    if (device === selectedAudioInputDevice) {
                        option.selected = true;
                    }
                });
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
let currentSamplePath = null;

// Initialize Wavesurfer.js when needed
async function initializeWaveform(containerId) {
    console.log(`🌊 Initializing waveform in container: ${containerId}`);
    
    try {
        // Import Wavesurfer dynamically
        const WaveSurfer = (await import('https://unpkg.com/wavesurfer.js@7/dist/wavesurfer.esm.js')).default;
        
        const container = document.getElementById(containerId);
        if (!container) {
            console.error(`❌ Waveform container not found: ${containerId}`);
            return null;
        }
        
        const wavesurfer = WaveSurfer.create({
            container: container,
            waveColor: '#4682B4',
            progressColor: '#dc2626',
            backgroundColor: '#1e1e1e',
            height: 128,
            normalize: true,
            fillParent: true,
            responsive: true
        });
        
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
    const containerElement = document.getElementById(isRangeMode ? 'range-waveform-container' : 'waveform-container');
    
    if (!containerElement) {
        console.error(`❌ Waveform container not found`);
        return;
    }
    
    // Show loading state
    const displayElement = document.getElementById(containerId);
    displayElement.innerHTML = '<div class="waveform-loading">Loading waveform...</div>';
    containerElement.style.display = 'block';
    
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

// Export waveform functions to global scope
window.zoomWaveform = zoomWaveform;
window.playWaveform = playWaveform;
window.playRangeWaveform = playRangeWaveform;
window.resetWaveformView = resetWaveformView;
window.showBatchThumbnails = showBatchThumbnails;
window.showWaveform = showWaveform;
window.hideWaveform = hideWaveform;

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
            monitorBtn.textContent = '🔴 Monitoring...';
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

// Populate the templates grid with available templates
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

// Test loop detection on the last recorded sample
async function testLoopDetection() {
    console.log('🔄 Testing loop detection on last recorded sample...');
    
    const statusElement = document.getElementById('loop-detection-status');
    const resultsElement = document.getElementById('loop-detection-results');
    
    if (statusElement) {
        statusElement.textContent = 'Running loop detection...';
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
        
        // Display results
        displayLoopDetectionResults(result, statusElement, resultsElement);
        
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
    const minLoopSlider = document.getElementById('min-loop-length');
    const maxLoopSlider = document.getElementById('max-loop-length');
    const correlationSlider = document.getElementById('correlation-threshold');
    
    return {
        minLoopLength: minLoopSlider ? parseFloat(minLoopSlider.value) : 0.1,
        maxLoopLength: maxLoopSlider ? parseFloat(maxLoopSlider.value) : 5.0,
        correlationThreshold: correlationSlider ? parseFloat(correlationSlider.value) : 0.8
    };
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
        
        if (statusElement) {
            if (loopResult.success) {
                statusElement.textContent = 'Loop detection completed successfully!';
                statusElement.style.color = '#16a34a';
            } else {
                statusElement.textContent = `Loop detection failed: ${loopResult.failure_reason || 'Unknown error'}`;
                statusElement.style.color = '#dc2626';
            }
        }
        
        if (resultsElement) {
            if (loopResult.success && loopResult.candidates && loopResult.candidates.length > 0) {
                let html = '<div class="loop-results-success">';
                html += `<h4>Found ${loopResult.candidates.length} loop candidate(s):</h4>`;
                
                loopResult.candidates.forEach((candidate, index) => {
                    const startSec = (candidate.start_sample / 44100).toFixed(3); // Assume 44.1kHz
                    const endSec = (candidate.end_sample / 44100).toFixed(3);
                    const lengthSec = (candidate.length_samples / 44100).toFixed(3);
                    
                    html += `
                        <div class="loop-candidate" style="margin: 10px 0; padding: 10px; border: 1px solid #374151; border-radius: 4px;">
                            <div><strong>Candidate ${index + 1}:</strong></div>
                            <div>Start: ${startSec}s (sample ${candidate.start_sample})</div>
                            <div>End: ${endSec}s (sample ${candidate.end_sample})</div>
                            <div>Length: ${lengthSec}s (${candidate.length_samples} samples)</div>
                            <div>Quality Score: ${candidate.quality_score.toFixed(3)}</div>
                            <div>Correlation: ${candidate.correlation.toFixed(3)}</div>
                            <div>Zero-crossing aligned: ${candidate.zero_crossing_aligned ? 'Yes' : 'No'}</div>
                        </div>
                    `;
                });
                
                if (loopResult.best_candidate) {
                    html += `<div style="margin-top: 15px; padding: 10px; background: rgba(34, 197, 94, 0.1); border-radius: 4px;">`;
                    html += `<strong>Best candidate:</strong> ${(loopResult.best_candidate.length_samples / 44100).toFixed(3)}s loop with quality ${loopResult.best_candidate.quality_score.toFixed(3)}`;
                    html += `</div>`;
                }
                
                html += '</div>';
                resultsElement.innerHTML = html;
                
            } else {
                resultsElement.innerHTML = `
                    <div style="color: #dc2626; padding: 10px;">
                        No suitable loop points found. Try adjusting the parameters or using a different sample.
                    </div>
                `;
            }
        }
        
        // Show success status
        if (loopResult.success) {
            showStatus(`Loop detection completed! Found ${loopResult.candidates.length} candidates.`, 'success');
        } else {
            showStatus(`Loop detection failed: ${loopResult.failure_reason || 'Unknown error'}`, 'error');
        }
        
    } catch (parseError) {
        console.error('❌ Failed to parse loop detection result:', parseError);
        
        if (statusElement) {
            statusElement.textContent = 'Failed to parse loop detection result';
            statusElement.style.color = '#dc2626';
        }
        
        if (resultsElement) {
            resultsElement.innerHTML = `<div style="color: #dc2626; padding: 10px;">Parse error: ${parseError}</div>`;
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