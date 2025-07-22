# Batcherbird Roadmap

*Open-source hardware sampling tool to rival expensive commercial solutions*

## 🎯 Vision Statement

Create the most professional, user-friendly hardware sampling tool available, combining the power of commercial samplers with the accessibility of open-source software.

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

---

## 🚀 **Next Release (v0.2 - Sampler Compatibility)**

*Priority: High | Timeline: Q2 2024*

### Enhanced Export Formats 🔄
- [ ] **SFZ format export** *(Priority 1)*
  - Open-source sampler format with wide compatibility
  - Text-based format for learning other sampler formats
  - Works in Kontakt, HALion, ARIA, and dedicated SFZ players
  - Full velocity layer and mapping support

- [ ] **Kontakt (.nki) file generation** *(Priority 2)*
  - Automatic instrument creation with velocity layers
  - Pre-configured sample zones and key mappings  
  - Root key detection from MIDI note numbers
  - Professional velocity curve mapping

- [ ] **Decent Sampler (.dspreset) XML export** *(Priority 3)*
  - Complete sample mapping definitions
  - Velocity layer XML structure
  - Root key and range specifications
  - Ready-to-load preset files

- [ ] **Additional sampler formats** *(Future)*
  - EXS24 (Logic Pro) support
  - HALion (Steinberg) support
  - Reason NN-XT format
  - Hardware sampler formats (Korg, etc.)

### Audio Monitoring
- [ ] **Real-time level meters**
  - Input level monitoring during recording
  - Peak and RMS level display
  - Clipping detection and warnings

---

## 🎵 **Future Releases**

### v0.3 - Advanced Sampling (Q3 2024)
- [ ] **Enhanced velocity layers** *(Professional parity)*
  - Support for up to 128 velocity layers (vs. current 4)
  - Automatic velocity range mapping across layers
  - Crossfade zones between velocity layers
  - MIDI Attack Velocity mapping reference

- [ ] **Loop point detection**
  - Automatic loop point suggestion
  - Manual loop point adjustment
  - Crossfade loop creation
  - Multiple loop modes and types

- [ ] **Advanced sample processing**
  - Automatic sample normalization options
  - Fade-in/fade-out processing
  - Sample trimming fine-tuning
  - Up/downsampling and channel re-mapping

- [ ] **Multi-sampling techniques**
  - Round-robin sample recording
  - Release trigger sampling
  - Sustain pedal sample capture

### v0.4 - Workflow Enhancement (Q4 2024)
- [ ] **Project Wizard** *(Professional guided workflow)*
  - Step-by-step guided sampling process
  - Beginner-friendly interface with expert tips
  - Automatic time and storage estimation
  - Project templates for common scenarios

- [ ] **VST Host Integration** *(Sample software instruments)*
  - Built-in VST2/VST3 host for sampling plugins
  - Offline bouncing for 80% time savings vs. real-time
  - Direct plugin parameter automation
  - Support for popular software synthesizers

- [ ] **Batch processing**
  - Instrument Copier - sample entire MIDI sound banks
  - Queue multiple instruments for unattended recording
  - Automatic patch changes via Program Change messages
  - Batch export to multiple formats

- [ ] **Template system**
  - Saveable recording templates
  - Instrument-specific configurations
  - Quick setup for common synthesizers
  - Preset Manager for common sampling scenarios

- [ ] **Project management**
  - Session saving and loading
  - Sample library organization
  - Project-based folder structure

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
- **v0.2**: SFZ export and improved format compatibility
- **v0.3**: Support for 128 velocity layers (professional parity)
- **v0.4**: VST host integration and automated workflows
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

### User Adoption
- GitHub stars and community engagement
- Professional musician testimonials
- Integration with popular DAWs and samplers

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
1. **Professional sampler file generation** - XML parsing and instrument creation
2. **XML sampler export** - Sample mapping XML generation
3. **Windows ASIO support** - Cross-platform audio driver integration
4. **Real-time level meters** - Audio visualization and monitoring

---

## 📞 **Contact & Community**

- **GitHub Issues**: Feature requests and bug reports
- **Documentation**: Technical architecture and API reference
- **Blog**: Development updates and technical deep-dives

---

*Batcherbird is committed to providing professional-quality sampling tools that are accessible, open-source, and community-driven. Our goal is to democratize music production by offering the same capabilities as expensive commercial tools, free for all musicians.*

**Next milestone: Professional sampler format export support** 🎹