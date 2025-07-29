# Product Requirements Document: Waveform Visualization

## Executive Summary
This PRD outlines the requirements for implementing real-time waveform visualization in the Batcherbird application. The feature will replace the current placeholder SVG waveform with actual audio waveform data from recorded samples, providing users with visual feedback and analysis capabilities for their recordings.

## Problem Statement
Currently, the Batcherbird application displays a static placeholder SVG waveform that does not represent the actual recorded audio. Users cannot:
- See the actual waveform of their recorded samples
- Analyze the audio visually for quality control
- Identify recording issues through visual inspection
- Preview specific sections of the recording
- Make informed decisions about loop points or sample trimming

## Goals and Objectives
1. **Primary Goal**: Display accurate waveform visualization of recorded audio samples
2. **Secondary Goals**:
   - Enable waveform-based playback controls
   - Support zoom and navigation functionality
   - Provide visual feedback during recording
   - Support both single and range recording modes

## User Stories
1. **As a sound designer**, I want to see the actual waveform of my recorded sample so I can verify the recording quality
2. **As a producer**, I want to zoom in on the waveform to inspect transients and zero-crossings for precise loop point selection
3. **As a musician**, I want to click on the waveform to preview audio from that position
4. **As a user**, I want to see the waveform update in real-time during recording to monitor input levels

## Functional Requirements

### Core Waveform Display
1. **Waveform Rendering**
   - Display stereo or mono waveforms based on recorded audio
   - Show peak amplitude representation
   - Support variable sample rates (44.1kHz, 48kHz, 96kHz, etc.)
   - Render efficiently for long recordings (up to 10 minutes)

2. **Data Pipeline**
   - Retrieve audio data from Rust backend after recording completion
   - Process audio data for visualization (downsampling for display)
   - Cache processed waveform data for performance
   - Update display when switching between recorded samples

3. **Visual Design**
   - Match the current dark theme aesthetic
   - Clear distinction between left/right channels for stereo
   - Visual indication of clipping or problematic areas
   - Maintain responsive design across different window sizes

### Interactive Features
1. **Playback Integration**
   - Click-to-seek functionality
   - Visual playhead indicator during playback
   - Highlight currently playing section

2. **Zoom Controls**
   - Zoom in/out buttons (already in UI)
   - Horizontal zoom for time axis
   - Vertical zoom for amplitude
   - Reset view button functionality

3. **Navigation**
   - Scroll/pan through zoomed waveform
   - Overview navigator for long samples
   - Keyboard shortcuts for navigation

### Recording Integration
1. **Live Waveform Updates**
   - Display waveform as it's being recorded (if feasible)
   - Show recording progress indicator
   - Visual feedback for input levels

2. **Post-Recording Updates**
   - Automatically load and display waveform after recording
   - Show processing indicator while generating waveform
   - Handle multiple recordings in range mode

## Technical Requirements

### Frontend (React/TypeScript)
1. **Waveform Component**
   - Create reusable React component for waveform display
   - Use Canvas API or WebGL for performance
   - Implement efficient rendering for large datasets
   - Support responsive resizing

2. **State Management**
   - Store waveform data in component state
   - Manage zoom level and viewport position
   - Track playback position and selection

3. **Data Processing**
   - Implement peak detection algorithm
   - Downsample audio data for display
   - Handle different audio formats (mono/stereo)

### Backend Integration (Rust/Tauri)
1. **Audio Data API**
   - New Tauri command: `get_waveform_data(file_path: string)`
   - Return format: peaks array, sample rate, duration, channels
   - Support streaming for large files

2. **Real-time Updates**
   - Event system for recording progress
   - Periodic waveform data updates during recording
   - Completion notification with final waveform data

3. **Performance Optimization**
   - Implement caching mechanism
   - Use efficient data structures
   - Minimize data transfer between backend and frontend

## Data Flow Architecture

```
Recording Flow:
1. User starts recording
2. Backend begins audio capture
3. Backend emits periodic audio level events
4. Frontend updates level meters (existing)
5. Recording completes
6. Backend processes audio file
7. Backend generates waveform data
8. Frontend requests waveform via Tauri command
9. Frontend renders waveform in UI

Playback Flow:
1. User clicks on waveform
2. Frontend calculates time position
3. Frontend calls backend playback command with position
4. Backend starts playback from position
5. Backend emits playback position events
6. Frontend updates playhead position
```

## Non-Functional Requirements

### Performance
- Waveform should render within 500ms of recording completion
- Smooth zooming and panning (60 fps)
- Minimal memory footprint for waveform data
- Support files up to 10 minutes without degradation

### Usability
- Intuitive zoom and navigation controls
- Clear visual feedback for all interactions
- Consistent with DAW waveform conventions
- Accessible keyboard navigation

### Compatibility
- Support all audio formats used by Batcherbird
- Work across different screen resolutions
- Maintain functionality in both development and production builds

## Implementation Phases

### Phase 1: Basic Waveform Display (MVP)
- Static waveform display after recording
- Replace placeholder SVG with real data
- Basic peak visualization
- Support for single recording mode

### Phase 2: Interactive Features
- Click-to-seek functionality
- Zoom in/out implementation
- Playback integration with visual feedback
- Pan/scroll navigation

### Phase 3: Advanced Features
- Real-time waveform during recording
- Range recording mode support
- Performance optimizations
- Advanced visualization options (RMS, spectral)

## Success Metrics
1. Waveform accurately represents recorded audio
2. All interactive features work reliably
3. Performance meets specified targets
4. User feedback indicates improved workflow

## Risks and Mitigation
1. **Performance with large files**
   - Mitigation: Implement progressive loading and efficient data structures

2. **Cross-platform rendering differences**
   - Mitigation: Use web standards (Canvas API) and test across platforms

3. **Backend data processing overhead**
   - Mitigation: Process waveform data asynchronously, show loading state

## Dependencies
- Existing Tauri backend audio processing
- React hooks for audio state management
- Canvas or WebGL rendering library (to be selected)
- Audio file reading capabilities in Rust backend

## Open Questions
1. Should we support waveform caching between sessions?
2. What level of zoom should be supported (sample-level)?
3. Should we implement waveform-based editing features?
4. Do we need spectral/frequency domain visualization?

## Appendix: Technical Specifications

### Waveform Data Format
```typescript
interface WaveformData {
  peaks: {
    positive: number[];
    negative: number[];
  };
  sampleRate: number;
  duration: number;
  channels: number;
  format: 'mono' | 'stereo';
}
```

### Tauri Commands
```rust
#[tauri::command]
async fn get_waveform_data(
    file_path: String,
    resolution: u32
) -> Result<WaveformData, String>

#[tauri::command]
async fn get_audio_segment(
    file_path: String,
    start_time: f64,
    end_time: f64
) -> Result<AudioSegment, String>
```

### React Component Interface
```typescript
interface WaveformProps {
  audioFile?: string;
  isRecording: boolean;
  onSeek?: (position: number) => void;
  zoomLevel?: number;
  playbackPosition?: number;
}
```