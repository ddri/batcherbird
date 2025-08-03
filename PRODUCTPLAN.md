# BatcherBird Product Plan

## Project Vision
BatcherBird is a professional desktop audio sampling application that enables musicians to record, process, and export high-quality samples from hardware synthesizers with real-time visual feedback and automated sample detection.

## Primary User
Musicians with hardware synthesizers (like Korg DW-6000) who want to:
- Record samples with real-time waveform visualization
- See immediate visual feedback during recording
- Seamlessly transition from recording to playback/editing
- Export samples in multiple formats (DecentSampler, SFZ, WAV)

## Current Status
- ✅ Basic MIDI integration and device connection
- ✅ Audio recording and file export functionality  
- ✅ **FIXED**: Real-time waveform visualization during recording (professional 3-thread architecture)
- ✅ Reliable waveform display after recording completes
- ✅ **NEW**: Professional seamless recording workflow following industry standards

## Product Epics

### Epic 1: Professional Real-Time Waveform Visualization ✅ **COMPLETED**
**Goal**: Implement industry-standard real-time waveform display during recording that follows professional audio application patterns.

**User Story**: As a musician recording my Korg DW-6000, I want to see a live waveform visualization while recording so I know the audio is being captured correctly and can monitor the recording quality in real-time.

**Technical Approach**: Follow INFRA-RESEARCH.md - implement 3-thread architecture with lock-free ring buffers and Tauri channels.

#### Milestones:
- [x] **M1.1**: Ring Buffer Audio Pipeline ✅ **COMPLETED**
  - ✅ Add rtrb ring buffer to existing CPAL audio callback
  - ✅ Verify no audio dropouts with lightweight stress test
  - ✅ Audio thread calculates basic peak/RMS data only
  - ✅ Created VizChunk struct with peak/RMS calculation
  - ✅ Integrated lock-free ring buffer into all audio callback formats (F32, I16, U16)
  - ✅ Added `sample_single_note_with_viz_blocking()` method that returns Consumer<VizChunk>

- [x] **M1.2**: Visualization Thread Implementation ✅ **COMPLETED**
  - ✅ Create dedicated visualization thread that reads from ring buffer at 60fps
  - ✅ Implement Tauri channel communication to frontend
  - ✅ Basic throughput testing (can handle 60fps data stream?)
  - ✅ Export VizChunk to Tauri frontend via serde serialization

- [x] **M1.3**: Frontend Visualization Replacement ✅ **COMPLETED**
  - ✅ Replace Web Audio API hook with Tauri channel listener
  - ✅ Implement Canvas-based real-time drawing using VizChunk data
  - ✅ Clean state management for recording start/stop

- [x] **M1.4**: Integration & Polish ✅ **COMPLETED**
  - ✅ Smooth recording-to-playback transition
  - ✅ Remove all Web Audio API dependencies  
  - ✅ Visual polish and performance optimization

**Success Criteria**: ✅ **ALL ACHIEVED**
- ✅ Real-time waveform displays during recording without audio dropouts
- ✅ Smooth 60fps visualization updates  
- ✅ No crashes or synchronization issues
- ✅ Clean transition from recording visualization to file-based waveform

### Epic 2: Seamless Recording Workflow ✅ **COMPLETED**
**Goal**: Perfect the end-to-end recording experience with the Korg DW-6000.

**User Story**: As a musician, I want to arm the recorder, trigger a note on my synth, see real-time feedback, and immediately play back what I recorded without any jarring transitions or UI glitches.

**Technical Approach**: Follow professional audio application patterns from Pro Tools, Logic Pro, and Ableton Live for seamless workflow optimization.

#### Milestones:
- [x] **M2.1**: Professional Count-In Timer and Visual Feedback ✅ **COMPLETED**
  - ✅ Added 2-second countdown before recording (Pro Tools/Logic pattern)
  - ✅ Professional circular progress indicator with smooth animations
  - ✅ Visual state transitions with proper color coding
  - ✅ ESC key cancellation support
  - ✅ Full-screen overlay that doesn't interrupt workflow

- [x] **M2.2**: Seamless Record→Playback Transition ✅ **COMPLETED**
  - ✅ Ableton Live-style "Stop & Play" button behavior
  - ✅ Immediate audio playback when stopping recording
  - ✅ Removed jarring 500ms delay in waveform loading
  - ✅ Background waveform loading for smooth visual transitions
  - ✅ Audio-first loading prioritizes immediate feedback

- [x] **M2.3**: Professional Keyboard Shortcuts ✅ **COMPLETED**
  - ✅ Spacebar for play/pause (universal professional standard)
  - ✅ ESC key for countdown cancellation
  - ✅ Smart input detection (prevents shortcuts when typing)
  - ✅ Console logging for debugging workflow

- [x] **M2.4**: Professional Error Handling ✅ **COMPLETED**
  - ✅ Replaced all alert() calls with professional notifications
  - ✅ Toast-style notifications with auto-dismiss
  - ✅ Error categorization (error, warning, success, info)
  - ✅ Professional messaging that doesn't interrupt workflow
  - ✅ Success notifications for positive feedback

**Success Criteria**: ✅ **ALL ACHIEVED**
- ✅ Professional countdown provides clear recording preparation
- ✅ Seamless stop-and-play transition matches Ableton Live experience
- ✅ Spacebar keyboard shortcut works universally for playback
- ✅ No jarring alert dialogs - all feedback is professional and contextual
- ✅ Recording workflow feels polished and professional

### Epic 3: Professional Audio Quality 
**Goal**: Transform BatcherBird into professional-grade sampling software with industry-standard audio processing and intelligent automation.

**User Story**: As a professional musician creating sample libraries, I want automated audio processing tools that match the quality and intelligence of commercial sampling software, with professional-grade level monitoring, automatic sample trimming, advanced loop detection, and optimized multi-format export.

**Technical Approach**: Implement professional audio algorithms with FFT-based processing, multi-threaded performance optimization, and industry-standard metadata support. See EPIC-3-DETAILED-PLAN.md for comprehensive technical specifications.

#### Epic 3.1: Professional Audio Level Monitoring and Gain Staging
**Duration**: 4 weeks | **Priority**: High
- **M3.1.1**: Professional Meter Engine (VU, PPM, LUFS with industry ballistics)
- **M3.1.2**: Gain Staging Assistant (-18dBFS target for synthesizer recording)
- **M3.1.3**: Professional Visual Meters (Pro Tools/Logic style interface)

#### Epic 3.2: Intelligent Sample Detection and Auto-Trimming  
**Duration**: 5 weeks | **Priority**: High
- **M3.2.1**: Multi-Algorithm Detection Engine (RMS + Spectral Flux + Phase Deviation)
- **M3.2.2**: Synthesizer-Specific Profiles (Pads, Leads, Percussive optimization)
- **M3.2.3**: Professional Trimming Engine (Attack preservation, quality validation)

#### Epic 3.3: Advanced Loop Detection and Professional Metadata
**Duration**: 5 weeks | **Priority**: Medium  
- **M3.3.1**: FFT-Based Loop Detection (5-10x performance improvement)
- **M3.3.2**: Professional Metadata Engine (SMPL chunk, Broadcast WAV)
- **M3.3.3**: Equal-Power Crossfading and Quality Optimization

#### Epic 3.4: Multi-Format Export Optimization and Professional Workflows
**Duration**: 6 weeks | **Priority**: Medium
- **M3.4.1**: Parallel Batch Processing Engine (4-8x speedup with rayon)
- **M3.4.2**: Advanced Sampler Format Support (Velocity layers, round-robin)
- **M3.4.3**: Professional Quality Validation and Workflow Integration

**Success Criteria**:
- Real-time processing without audio dropouts (<5ms latency for monitoring)
- 95%+ detection accuracy across synthesizer types
- 100% compatibility with major DAWs and samplers
- Professional-quality results indistinguishable from commercial sample libraries
- 4-8x performance improvement for batch operations

**Technical Dependencies**:
- `rustfft` for FFT-based correlation and spectral analysis
- `rayon` for parallel batch processing  
- `spectrum-analyzer` for real-time spectral analysis
- `wide` for SIMD optimization

### Epic 4: Advanced Sampling Features (FUTURE)
**Goal**: Power-user features for advanced sampling workflows.

#### Potential Milestones:
- Velocity layer recording automation
- Note range recording with progress tracking
- Template saving/loading
- Batch processing capabilities

## Development Workflow

### Current Epic Execution Process:
1. **Reference PRODUCTPLAN.md** for current milestone and context
2. **Check INFRA-RESEARCH.md** for technical approach and patterns
3. **Implement** with architecture validation checkpoints
4. **Test** with Korg DW-6000 in real recording scenarios
5. **Update** PRODUCTPLAN.md with progress and learnings

### Epic Completion Criteria:
- All milestones completed successfully
- Real-world testing with Korg DW-6000 passes
- No regressions in existing functionality
- User story goal achieved

## Test Hardware Configuration
- **Primary Synth**: Korg DW-6000 (analog/digital hybrid, 6-voice polyphony)
- **Audio Interface**: [To be specified]
- **Recording Scenarios**: 
  - Single note recording (C4, various velocities)
  - Sustained pad sounds with long release
  - Short percussive sounds
  - Full velocity layer recordings

## Success Metrics
- **Zero audio dropouts** during recording
- **Sub-20ms latency** for real-time visualization
- **Smooth 60fps** waveform updates
- **Crash-free** recording sessions
- **Professional audio quality** output files

---

**Next Action**: Begin Epic 3 - Professional Audio Quality

**Status**: 🎉 **EPIC 2 COMPLETED!** Professional seamless recording workflow is now fully implemented following industry standards from Pro Tools, Logic Pro, and Ableton Live. Features include:

- **Professional Count-In**: 2-second countdown with visual progress indicator
- **Seamless Playback**: Ableton-style "Stop & Play" button for immediate feedback  
- **Keyboard Shortcuts**: Spacebar for play/pause following universal standards
- **Professional Notifications**: Toast-style error/success feedback replacing jarring alerts

The recording experience now feels polished and professional, matching the workflow patterns users expect from industry-leading DAWs. Ready to proceed with Epic 3 for professional audio quality improvements.