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

### Epic 3: Professional Audio Quality (FUTURE)
**Goal**: Ensure recording quality meets professional standards.

#### Potential Milestones:
- Audio level monitoring and gain staging
- Sample detection and auto-trimming
- Loop detection and metadata
- Multiple export format optimization

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