# Batcherbird Roadmap

*Open-source hardware sampling tool*

## 🎯 Vision Statement

Create a user-friendly hardware sampling tool for everyone, combining the power of commercial samplers with the accessibility of open-source software.

---

## ✅ **Current Status (v0.1 - Foundation)**

### Core Audio Engine ✅
- [x] Tauri 2.0 + Rust backend with blocking audio architecture
- [x] CPAL cross-platform audio support with thread safety
- [x] Persistent audio streams (Ableton-style) for reliable recording
- [x] Professional 32-bit float WAV export

### MIDI Integration ✅  
- [x] Cross-platform MIDI device detection and connection
- [x] Note preview with customizable velocity and duration
- [x] Enhanced MIDI panic for vintage synthesizers (DW6000 tested)
- [x] Channel-specific and full panic modes

### Recording Features ✅
- [x] Single note sampling with progress tracking
- [x] Range sampling (C2-C7) with real-time progress
- [x] Working stop button with responsive UI
- [x] Individual note recording for accurate progress display

### Sample Processing ✅
- [x] Professional RMS window analysis for automatic sample detection
- [x] Smart presets (Vintage Synth, Percussive, Sustained/Pads)
- [x] Configurable threshold and timing parameters
- [x] Graceful fallback when detection fails

### Velocity Layer Sampling ✅
- [x] Multiple velocity layer recording (2/3/4 layers + custom)
- [x] Professional velocity curves for realistic dynamics
- [x] Smart UI that adapts between single/multi-velocity modes
- [x] Comprehensive progress tracking for velocity sessions

### Professional Organization ✅
- [x] Professional automatic folder organization
- [x] Consistent velocity naming (vel064, vel127 format)
- [x] Sample name prefixing for instrument organization
- [x] Industry-standard filename structure

### User Experience ✅
- [x] Native macOS directory picker integration
- [x] Persistent preferences across sessions
- [x] Real-time progress tracking and status updates
- [x] Professional dark theme UI
- [x] Comprehensive error handling and user feedback

### Waveform Visualization ✅
- [x] Post-recording waveform visualization with Wavesurfer.js integration
- [x] Interactive playback controls (play/pause, zoom, reset)
- [x] Tauri 2.0 asset protocol integration for secure file access
- [x] Range recording waveform display with real-time updates
- [x] Sample information display (duration, note, velocity)
- [x] Professional dark theme integration

### Real-time Audio Monitoring ✅
- [x] Professional VU-style meters with peak and RMS display
- [x] Color-coded zones: Green (good), Yellow (loud), Red (clipping)
- [x] AKAI-style explicit "Monitor Input" toggle button
- [x] Peak hold functionality with decay animation
- [x] Clipping detection with visual warnings
- [x] Professional broadcast-standard meter ballistics
- [x] Thread-safe audio level detection using SamplingEngine infrastructure
- [x] 30 Hz UI refresh rate for smooth visual feedback

### Sampler Format Support ✅
- [x] **Decent Sampler (.dspreset) XML export**
  - Complete sample mapping definitions with velocity layers
  - Professional UI controls (Attack, Release, Tone, Reverb)
  - Creator metadata and instrument descriptions
  - Automatic velocity range distribution across layers
  - Ready-to-load preset files for immediate use

---

## 🚀 **Next Release (v0.2 - Professional Automation)**

*Priority: High | Timeline: Q1 2025*

### Core Automation Features 🔄
- [ ] **Intelligent Auto-Loop Detection** *(Priority 1)*
  - Multiple algorithm modes: Percussive, Sustained, Harmonic
  - Zero-crossing detection with spectral analysis
  - Automatic crossfade optimization for seamless loops
  - User preview and fine-tuning of suggested loop points
  - Preserves musicality while eliminating manual tedium
  - Industry-leading loop quality comparable to professional tools

- [ ] **One-Click Full Instrument Sampling** *(Priority 2)*
  - "Sample Complete Instrument" button for hands-off operation
  - Automatic note range detection (C2-C6 default, user configurable)
  - Multiple velocity layer automation (1-16 layers)
  - Intelligent MIDI sequencing with optimal timing
  - Real-time progress: "Recording C#3, velocity 80 (47/127)"
  - Reduces 12+ hour manual process to automated workflow

### Enhanced Export Formats 🔄
- [ ] **SFZ format export** *(Priority 3)*
  - Open-source sampler format with universal compatibility
  - Text-based format ideal for learning and customization
  - Works in Kontakt, HALion, ARIA, sforzando, LinuxSampler
  - Full velocity layer and mapping support
  - Perfect stepping stone to other sampler formats

- [ ] **Kontakt (.nki) file generation** *(Priority 4)*
  - Professional-grade instrument creation
  - Pre-configured sample zones and key mappings  
  - Root key detection from MIDI note numbers
  - Advanced velocity curve mapping
  - Industry standard for professional sample libraries

### Professional Workflow Features 🔄

- [ ] **Project Wizard & Templates** *(Priority 5)*
  - "Vintage Analog Synth" template (C2-C6, 3 velocity layers, 2s duration)
  - "Electric Piano" template (A0-C8, 4 velocity layers, 8s with release)
  - "Drum Machine" template (36-81, single velocity, 0.5s duration)
  - "Hardware Synthesizer" auto-detection and optimal settings
  - Guided workflow for beginners with expert tips
  - Automatic time and storage estimation

- [ ] **Post-Recording Export Customization Suite** *(Priority 6)*
  - Interactive sample assignment editor (reorder, exclude, combine samples)
  - Velocity curve editor for remapping recorded velocities to target ranges
  - Round-robin group configuration for natural variations
  - Custom envelope settings per sampler format (Attack, Release, etc.)
  - Preview generated instrument before final export
  - Batch export to multiple formats simultaneously
  - Format-specific metadata editor (creator info, descriptions, categories)
  - Sample loop point adjustment and crossfade editor

- [ ] **Advanced Audio Processing** *(Priority 7)*
  - Automatic sample trimming with intelligent silence detection
  - Professional noise gate with adjustable threshold
  - Smart normalization preserving dynamics
  - Phase alignment for stereo sources
  - Quality control with automatic re-recording of failed samples

---

## 🎵 **Future Releases**

### v0.3 - Hardware Integration Excellence (Q2 2025)
- [ ] **Hardware Sampling as a Service** *(Unique competitive advantage)*
  - Automatic MIDI device discovery and optimal settings suggestion
  - Built-in hardware profiles for popular synthesizers (DW6000, Prophet 5, etc.)
  - Intelligent sampling that adapts to hardware response characteristics
  - Quality control with automatic detection and re-recording of failed samples
  - "Sample my entire studio" batch processing capabilities

- [ ] **Advanced Loop Technology**
  - Multiple loop modes preserving musicality (Short, Long, Complex)
  - Spectral analysis for optimal loop point detection
  - Crossfade calculation with anti-aliasing
  - Loop validation and smoothing algorithms
  - Real-time loop preview and adjustment

- [ ] **Batch Processing Engine**
  - Queue multiple instruments for unattended overnight sampling
  - Background processing with progress tracking
  - Error handling and retry logic for failed recordings
  - Multiple format export pipeline
  - Comprehensive logging and reporting

- [ ] **Enhanced Visual Feedback**
  - Batch thumbnail view with quality assessment
  - Interactive waveform editing with drag-to-adjust boundaries
  - Spectral analysis integration for frequency content
  - Velocity layer visualization and comparison tools
  - Real-time recording progress with visual waveforms

### v0.4 - Professional Studio Integration (Q3 2025)
- [ ] **VST Host Integration** *(Sample software instruments)*
  - Built-in VST2/VST3 host for sampling plugins
  - Offline bouncing for 80% time savings vs. real-time
  - Direct plugin parameter automation and modulation capture
  - Support for popular software synthesizers
  - Multi-timbral instrument sampling

- [ ] **Advanced Multi-Sampling**
  - Round-robin sample recording (up to 16 variations per note)
  - Release trigger sampling for realistic instrument behavior
  - Sustain pedal sample capture for piano-style instruments
  - Aftertouch and pitch bend recording
  - Multiple MIDI channel recording

- [ ] **Professional Sample Library Management**
  - Project-based organization with metadata tagging
  - Sample library browser with search and filtering
  - Automatic sample analysis and cataloging
  - Cross-reference tracking for multi-format exports
  - Backup and restoration capabilities

- [ ] **Enhanced Velocity Layer Support**
  - Support for up to 128 velocity layers
  - Automatic velocity range mapping with crossfade zones
  - Velocity curve visualization and editing
  - Layer blending and morphing capabilities
  - Professional dynamics analysis and optimization

### v0.5 - Pro Features (2025)
- [ ] **Advanced MIDI features**
  - Multiple MIDI channel recording
  - CC modulation capture
  - Aftertouch and pitch bend recording

- [ ] **Sample analysis**
  - Frequency analysis and EQ suggestions
  - Amplitude envelope visualization
  - Sample comparison tools

- [ ] **Cloud integration**
  - Sample library sync
  - Community sample sharing
  - Automatic backups

---

## 🛠️ **Technical Roadmap**

### Architecture Evolution
- **Current**: Tauri 2.0 + Rust backend with blocking audio
- **Future**: Plugin architecture for custom processing
- **Long-term**: Real-time analysis and visualization

### Performance Targets
- **v0.2**: Intelligent auto-loop detection and one-click instrument sampling
- **v0.3**: Hardware integration excellence with batch processing
- **v0.4**: VST host integration and advanced multi-sampling
- **v0.5**: Sub-10ms MIDI-to-audio latency and real-time processing

### Platform Support
- **Current**: macOS (primary development platform)
- **v0.2**: Windows support with ASIO
- **v0.3**: Linux support with ALSA/JACK
- **v0.4**: Hardware-optimized builds for each platform

---

## 🎯 **Success Metrics**

### Technical Excellence
- Zero audio dropouts during recording sessions
- < 1% sample detection false positives
- Professional-quality exports comparable to commercial tools

### Community Impact
- Open-source contributions and pull requests
- Educational content and documentation
- Hardware synthesizer compatibility database

---

## 🤝 **Contributing**

Batcherbird welcomes contributions in all areas:

- **Audio Engineers**: Sample detection algorithms, audio processing
- **UI/UX Designers**: User interface improvements and workflow optimization  
- **Musicians**: Hardware testing, feature requests, workflow feedback
- **Developers**: Cross-platform support, performance optimization

### Current Priorities for Contributors
1. **Intelligent auto-loop detection** - Multiple algorithm modes with spectral analysis for professional-quality loops
2. **One-click instrument sampling** - Full automation of the sampling process from MIDI sequencing to file organization
3. **SFZ format export** - Universal sampler compatibility with text-based format
4. **Project wizard and templates** - Guided workflows for common sampling scenarios
5. **Advanced audio processing** - Automatic trimming, normalization, and quality control
6. **Windows ASIO support** - Cross-platform audio driver integration
7. **Hardware integration profiles** - Optimized settings for popular synthesizers

---

*Batcherbird is committed to providing professional-quality sampling tools that are accessible, open-source, and community-driven. Our goal is to democratize music production by offering the same time-saving automation and professional results as expensive commercial tools, free for all musicians.*


**Next milestone: Intelligent Auto-Loop Detection & One-Click Sampling** 🤖