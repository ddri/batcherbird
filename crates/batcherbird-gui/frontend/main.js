const { invoke } = window.__TAURI__.core;

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
        console.log('📡 Calling backend record_sample with params:', { note, velocity, duration, outputDirectory, sampleName });
        
        try {
            const result = await invoke('record_sample', { 
                note: note, 
                velocity: velocity, 
                duration: duration,
                outputDirectory: outputDirectory,
                sampleName: sampleName || null
            });
            console.log('✅ Backend returned result:', result);
            
            // Update UI with success
            recordingText.textContent = 'Recording complete!';
            showStatus(result, 'success');
            
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
        
        // Record notes using async scheduler to keep UI responsive (Ableton-style)
        await recordNotesWithVelocityLayersResponsiveUI(startNote, endNote, velocities, duration, outputDirectory, sampleName,
            successfulRecordings, totalSamples, rangeProgressFill, rangeRecordingText, rangeCurrentNote, rangeVelocityInfo, noteToName);
        
        // Update successfulRecordings from the recording process
        successfulRecordings = window.rangeRecordingResults ? window.rangeRecordingResults.successfulRecordings : 0;
        
        // Final UI update
        if (isRangeRecording) {
            rangeProgressFill.style.width = '100%';
            if (successfulRecordings === totalSamples) {
                rangeRecordingText.textContent = 'Range recording complete!';
                rangeCurrentNote.textContent = `✅ Completed ${successfulRecordings} of ${totalSamples} samples successfully`;
                showStatus(`Range recording complete! ${successfulRecordings} samples saved.`, 'success');
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
        });
    }
    
    // Setup value display updates for range inputs
    setupRangeInputs();
    
    // Setup velocity layer controls
    setupVelocityLayerControls();
    
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
                case 'kontakt':
                    exampleText = `Example: ${sampleName}/${sampleName}_C4_60_vel127.wav + .nki`;
                    break;
                case 'decentsampler':
                    exampleText = `Example: ${sampleName}/${sampleName}_C4_60_vel127.wav + .dspreset`;
                    break;
                case 'all':
                    exampleText = `Example: ${sampleName}/ + WAV/NKI/DSPRESET files`;
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
                // Record individual sample with specific velocity
                const result = await invoke('record_sample', { 
                    note: currentNote, 
                    velocity: velocity, 
                    duration: duration,
                    outputDirectory: outputDirectory,
                    sampleName: sampleName || null
                });
                
                console.log(`✅ Sample ${noteName} vel ${velocity} recorded successfully: ${result}`);
                window.rangeRecordingResults.successfulRecordings++;
                
                // Update current note to show success
                rangeCurrentNote.textContent = `✅ ${noteName} (${currentNote}) vel ${velocity} recorded`;
                
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
window.sendMidiPanic = sendMidiPanic;
window.openSetupModal = openSetupModal;
window.closeSetupModal = closeSetupModal;
window.switchRecordingMode = switchRecordingMode;
window.updateStatusBar = updateStatusBar;

// Debug: Verify functions are available
console.log('🔧 Functions exported to window:', {
    loadMidiDevices: typeof window.loadMidiDevices,
    testMidiConnection: typeof window.testMidiConnection,
    previewNote: typeof window.previewNote,
    recordSample: typeof window.recordSample
});