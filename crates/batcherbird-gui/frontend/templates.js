// Project Templates for Common Sampling Scenarios
// Each template configures optimal settings for different instrument types

const PROJECT_TEMPLATES = {
    'vintage_analog_synth': {
        name: 'Vintage Analog Synth',
        description: 'Perfect for DW6000, Prophet 5, Juno-106, and similar analog synthesizers',
        icon: '🎛️',
        settings: {
            // Recording settings
            recordingMode: 'range',
            startNote: 36,  // C2
            endNote: 84,    // C6
            velocityLayers: true,
            velocityPreset: '3',  // 3 velocity layers (soft, medium, loud)
            customVelocities: '48,96,127',  // Musical velocity distribution
            duration: 3000,  // 3 seconds - enough for analog filter sweeps
            
            // Export settings  
            exportFormat: 'sfz',  // Universal compatibility
            sampleName: '',  // User will fill this
            
            // Detection settings
            detectionEnabled: true,
            detectionPreset: 'vintage_synth',
            detectionThreshold: -35,
            
            // Metadata
            creatorName: '',
            instrumentDescription: 'Vintage analog synthesizer with rich harmonic content'
        },
        estimatedTime: '15-20 minutes',
        estimatedSamples: 147  // 49 notes × 3 velocities
    },
    
    'electric_piano': {
        name: 'Electric Piano',
        description: 'Rhodes, Wurlitzer, CP70 - capture the full sustain and release',
        icon: '🎹',
        settings: {
            // Recording settings
            recordingMode: 'range', 
            startNote: 36,  // C2 - limited by current UI
            endNote: 96,    // C7 - full available range
            velocityLayers: true,
            velocityPreset: '4',  // 4 velocity layers for expressive playing
            customVelocities: '32,64,96,127',
            duration: 8000,  // 8 seconds - capture full sustain and release
            
            // Export settings
            exportFormat: 'sfz',
            sampleName: '',
            
            // Detection settings
            detectionEnabled: true,
            detectionPreset: 'sustained',  // Better for long decays
            detectionThreshold: -50,  // Capture longer tail
            
            // Metadata
            creatorName: '',
            instrumentDescription: 'Electric piano with natural sustain and release'
        },
        estimatedTime: '45-60 minutes',
        estimatedSamples: 244  // 61 notes × 4 velocities (C2-C7)
    },
    
    'drum_machine': {
        name: 'Drum Machine',
        description: 'TR-808, TR-909, LinDrum - percussive sounds with tight timing',
        icon: '🥁',
        settings: {
            // Recording settings
            recordingMode: 'range',
            startNote: 36,  // C2 - standard drum range start
            endNote: 81,    // A5 - covers most drum machines
            velocityLayers: false,  // Most drum machines are velocity-sensitive but single-layer works
            velocityPreset: '1',
            customVelocities: '127',  // Full velocity
            duration: 2000,  // 2 seconds - enough for decay
            
            // Export settings
            exportFormat: 'sfz',
            sampleName: '',
            
            // Detection settings
            detectionEnabled: true,
            detectionPreset: 'percussive',
            detectionThreshold: -30,  // Tight detection for drums
            
            // Metadata
            creatorName: '',
            instrumentDescription: 'Drum machine with punchy, tight sounds'
        },
        estimatedTime: '8-12 minutes',
        estimatedSamples: 46  // 46 notes × 1 velocity
    },
    
    'pad_strings': {
        name: 'Pads & Strings',
        description: 'Lush pads, string machines, ambient textures',
        icon: '🌊',
        settings: {
            // Recording settings
            recordingMode: 'range',
            startNote: 36,  // C2 - lowest available
            endNote: 96,    // C7 - high range for sparkle
            velocityLayers: true,
            velocityPreset: '2',  // 2 layers - pads are usually more about timbre than dynamics
            customVelocities: '64,127',
            duration: 6000,  // 6 seconds - pads need time to evolve
            
            // Export settings
            exportFormat: 'sfz',
            sampleName: '',
            
            // Detection settings
            detectionEnabled: true,
            detectionPreset: 'sustained',
            detectionThreshold: -45,  // Capture subtle tails
            
            // Metadata
            creatorName: '',
            instrumentDescription: 'Lush pad with evolving textures and harmonic richness'
        },
        estimatedTime: '25-35 minutes',
        estimatedSamples: 122  // 61 notes × 2 velocities (C2-C7)  
    },
    
    'bass_synth': {
        name: 'Bass Synthesizer',
        description: 'Moog, TB-303, analog bass - focus on low-end power',
        icon: '🎸',
        settings: {
            // Recording settings
            recordingMode: 'range',
            startNote: 36,  // C2 - lowest available
            endNote: 60,    // C4 - bass range
            velocityLayers: true, 
            velocityPreset: '3',  // 3 layers for bass dynamics
            customVelocities: '48,96,127',
            duration: 4000,  // 4 seconds - bass notes need sustain
            
            // Export settings
            exportFormat: 'sfz',
            sampleName: '',
            
            // Detection settings
            detectionEnabled: true,
            detectionPreset: 'vintage_synth',
            detectionThreshold: -35,
            
            // Metadata
            creatorName: '',
            instrumentDescription: 'Analog bass synthesizer with deep, rich low-end'
        },
        estimatedTime: '18-25 minutes',
        estimatedSamples: 75   // 25 notes × 3 velocities (C2-C4)
    }
};

// Template utility functions
function getTemplateById(templateId) {
    return PROJECT_TEMPLATES[templateId] || null;
}

function getAllTemplates() {
    return Object.keys(PROJECT_TEMPLATES).map(id => ({
        id,
        ...PROJECT_TEMPLATES[id]
    }));
}

function applyTemplate(templateId) {
    const template = getTemplateById(templateId);
    if (!template) {
        console.error('Template not found:', templateId);
        return false;
    }
    
    console.log('🎯 Applying template:', template.name);
    
    try {
        // Apply recording mode
        switchRecordingMode(template.settings.recordingMode);
        
        // Apply range settings if in range mode
        if (template.settings.recordingMode === 'range') {
            const startNoteSelect = document.getElementById('start-note-select');
            const endNoteSelect = document.getElementById('end-note-select');
            
            if (startNoteSelect) startNoteSelect.value = template.settings.startNote;
            if (endNoteSelect) endNoteSelect.value = template.settings.endNote;
        }
        
        // Apply velocity settings
        const velocityLayersEnabled = document.getElementById('velocity-layers-enabled');
        const velocityLayersPreset = document.getElementById('velocity-layers-preset');
        const velocityLayersCustom = document.getElementById('velocity-layers-custom');
        
        if (velocityLayersEnabled) {
            velocityLayersEnabled.checked = template.settings.velocityLayers;
            velocityLayersEnabled.dispatchEvent(new Event('change'));
        }
        
        if (velocityLayersPreset) {
            velocityLayersPreset.value = template.settings.velocityPreset;
            velocityLayersPreset.dispatchEvent(new Event('change'));
        }
        
        if (velocityLayersCustom) {
            velocityLayersCustom.value = template.settings.customVelocities;
        }
        
        // Apply duration
        const durationInput = document.getElementById('range-duration-input');
        const durationDisplay = document.getElementById('range-duration-display');
        
        if (durationInput && durationDisplay) {
            durationInput.value = template.settings.duration;
            durationDisplay.textContent = template.settings.duration;
        }
        
        // Apply export format
        const exportFormatSelect = document.getElementById('export-format');
        if (exportFormatSelect) {
            exportFormatSelect.value = template.settings.exportFormat;
            exportFormatSelect.dispatchEvent(new Event('change'));
        }
        
        // Apply detection settings
        const detectionEnabled = document.getElementById('detection-enabled');  
        const detectionPreset = document.getElementById('detection-preset');
        const detectionThreshold = document.getElementById('detection-threshold');
        const detectionThresholdDisplay = document.getElementById('detection-threshold-display');
        
        if (detectionEnabled) {
            detectionEnabled.checked = template.settings.detectionEnabled;
            detectionEnabled.dispatchEvent(new Event('change'));
        }
        
        if (detectionPreset) {
            detectionPreset.value = template.settings.detectionPreset;
            detectionPreset.dispatchEvent(new Event('change'));
        }
        
        if (detectionThreshold && detectionThresholdDisplay) {
            detectionThreshold.value = template.settings.detectionThreshold;
            detectionThresholdDisplay.textContent = template.settings.detectionThreshold;
        }
        
        // Apply metadata
        const instrumentDescription = document.getElementById('instrument-description');
        if (instrumentDescription) {
            instrumentDescription.value = template.settings.instrumentDescription;
        }
        
        // Save preferences
        if (typeof savePreferences === 'function') {
            savePreferences();
        }
        
        // Show success message
        if (typeof showStatus === 'function') {
            showStatus(`Applied template: ${template.name} (${template.estimatedSamples} samples, ~${template.estimatedTime})`, 'success');
        }
        
        console.log('✅ Template applied successfully');
        return true;
        
    } catch (error) {
        console.error('❌ Failed to apply template:', error);
        if (typeof showStatus === 'function') {
            showStatus(`Failed to apply template: ${error.message}`, 'error');
        }
        return false;
    }
}

// Export for use in main.js
if (typeof window !== 'undefined') {
    window.PROJECT_TEMPLATES = PROJECT_TEMPLATES;
    window.getTemplateById = getTemplateById;
    window.getAllTemplates = getAllTemplates;
    window.applyTemplate = applyTemplate;
}

// Export for Node.js if needed
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        PROJECT_TEMPLATES,
        getTemplateById,
        getAllTemplates,
        applyTemplate
    };
}