# 🎹 Batcherbird

**Professional auto-sampling tool for hardware synthesizers**

Batcherbird is an open-source hardware sampling tool built with Rust and Tauri. It provides professional-grade batch sampling of hardware synthesizers with pristine audio quality, offering an accessible alternative to expensive commercial solutions.

![Batcherbird Screenshot](screenshot.png)

## ✨ Features

### 🎵 **Professional Audio Quality**
- 32-bit float WAV export (studio standard)
- Sub-millisecond MIDI timing precision
- Automatic release tail capture (500ms professional standard)
- Zero-dropout recording engine with persistent streams
- **Smart Sample Detection**: Automatic trimming with RMS window analysis

### 🎹 **Hardware Synthesizer Support**
- Auto-detects MIDI and audio interfaces
- Tested with real hardware (Korg DW6000, Arturia MiniFuse)
- Enhanced MIDI panic for vintage synthesizer compatibility
- Professional timing delays optimized for hardware stability

### 🚀 **Advanced Recording Modes**
- **Single Note Recording**: Precise individual note sampling with custom velocity
- **Range Recording**: Batch sample entire octaves (C2-C7) automatically
- **Velocity Layer Sampling**: Multi-dynamic recording (2/3/4 layers + custom)
  - Professional velocity curves (pp/mp/mf/ff dynamics)
  - Smart naming: `DW6000_C4_60_vel127.wav`
  - Crossfade-ready velocity mapping
- **Real-time Progress**: Sample-accurate progress with current note display

### 📊 **Professional Level Metering**
- **Lock-Free Architecture**: 60fps real-time meters with zero audio dropouts
- **Industry Standards**: Matches Pro Tools/Logic/Ableton metering
- **Multiple Meter Types**: Peak, RMS, and Peak Hold indicators
- **Professional Ballistics**: VU-style RMS with 11ms window
- **Visual Design**: dB scale (-60 to 0), color-coded zones, clipping indicators
- See [REALTIME_METERS.md](REALTIME_METERS.md) for technical details

### 🎨 **Professional UI Design**
- **Status Bar**: Real-time device connection indicators
- **Recording-First Layout**: Main controls prominently featured
- **Settings Sidebar**: Organized secondary options
- **Modal Device Setup**: Clean, focused configuration
- **Visual Feedback**: Color-coded status (Green=Connected, Red=Recording)

### 🗂️ **Smart File Organization**
- **Professional Folder Structure**: Automatic instrument subfolders
- **Industry-Standard Naming**: Both note name (C4) and MIDI number (60)
- **Consistent Velocity Format**: Always use `vel064`, `vel127` format
- **Sampler-Ready Format**: Prepared for professional sampler import

### 🛠️ **Professional Workflow**
- Native macOS folder picker integration
- Persistent preferences across sessions
- Session-safe MIDI state management
- Comprehensive error handling and graceful recovery

## 🔧 Installation & Setup

### Prerequisites
- macOS 10.15+ (Catalina or later)
- Audio interface connected to your synthesizer
- MIDI connection to your synthesizer
- Node.js 18+ and Rust 1.70+ (for development)

### Download
1. Download the latest release from [Releases](https://github.com/yourusername/batcherbird/releases)
2. Open the `.dmg` file and drag Batcherbird to Applications
3. Grant microphone and MIDI permissions when prompted

### Development Setup

#### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (via Homebrew)
brew install node

# Install Tauri CLI
cargo install tauri-cli --version "^2.0"
```

#### Clone & Setup
```bash
git clone https://github.com/yourusername/batcherbird.git
cd batcherbird
cd crates/batcherbird-gui
npm install
```

#### Starting the Application

**🚀 Quick Start (Recommended):**
```bash
cd crates/batcherbird-gui
npm run dev
```
This starts both the Tauri backend and frontend development server in one command.

**🔧 Alternative: Separate Processes**

If you need to start them separately:

*Backend only:*
```bash
cd crates/batcherbird-gui
npm run tauri dev
```

*Frontend only:*
```bash
cd crates/batcherbird-gui  
npm run dev:frontend
```

#### What Each Process Does

**Tauri Process (Backend)**: 
- Rust backend with professional audio processing
- MIDI device management and real-time communication
- File system operations and sample export
- Native desktop window and system integration

**Vite Process (Frontend)**:
- React TypeScript UI with professional meter displays
- Real-time waveform visualization and audio monitoring
- User interface controls and device configuration
- Hot reload for development

#### Hardware Setup
Make sure you have:
- ✅ Audio interface connected (or built-in audio)
- ✅ MIDI device connected (like your Korg DW-6000)
- ✅ Audio cables properly routed (synth → interface → Mac)
- ✅ MIDI cables connected (Mac → MIDI interface → synth)

The app will auto-detect available MIDI and audio devices on startup.

#### Production Build
```bash
cd crates/batcherbird-gui
cargo tauri build
```

## 🎯 Quick Start

### 1. **Connect Your Hardware**
   - **Audio**: Synth → Audio Interface → Mac
   - **MIDI**: Mac → MIDI Interface → Synth

### 2. **Launch Batcherbird**
The new professional interface shows:
   - **Status Bar**: Connection indicators at the top
   - **Recording Panel**: Main controls in center
   - **Settings Sidebar**: Output and detection options on right

### 3. **Setup Devices**
   - Click **⚙️ Setup** to open device configuration
   - Select your **MIDI Output** device (to trigger notes)
   - Select your **Audio Input** device (to record synth output)
   - Select your **Audio Output** device (for monitoring)
   - Click **Test** to verify MIDI communication
   - Click **Done** - status bar will show green indicators when connected

### 4. **Configure Output**
In the **Settings Sidebar**:
   - **Sample Name**: Enter your instrument name (e.g., "DW6000")
   - **Save Location**: Choose folder or use default Desktop/Batcherbird Samples
   - **Auto-trim samples**: Enable for automatic sample detection

### 5. **Record Samples**

#### **🎵 Single Note Mode**
   - Select **Single Note** tab
   - Choose note (C4, C3, etc.)
   - Set velocity (1-127) and duration
   - Click **🔴 Record Sample**

#### **🎹 Range Recording Mode**
   - Select **Range Recording** tab
   - Set note range (e.g., C4 to C6)
   - **For Single Velocity**: Leave "Velocity Layers" unchecked
   - **For Multi-Velocity**: Check "Velocity Layers" and choose:
     - **2 Layers**: Soft (64) + Loud (127)
     - **3 Layers**: Soft (48) + Medium (96) + Loud (127)  
     - **4 Layers**: Very Soft (32) + Soft (64) + Medium (96) + Loud (127)
     - **Custom**: Enter comma-separated values (e.g., "40,80,120")
   - Click **🎹 Record Range**

### 6. **Monitor Progress**
   - **Real-time progress bar** shows completion percentage
   - **Current note display** shows which note is being recorded
   - **Velocity info** shows current layer (for multi-velocity)
   - **Stop button** appears during recording for immediate cancellation

### 7. **Access Your Samples**
   - Click **Show in Finder** to open your samples folder
   - Files are organized in instrument subfolders: `Desktop/Batcherbird Samples/DW6000/`
   - Professional naming: `DW6000_C4_60_vel127.wav`

## 🎛️ Recording Modes Deep Dive

### 🎵 **Single Note Recording**
Perfect for:
- Testing individual patches
- Recording specific notes for loops or one-shots
- Quick sampling workflow

**Workflow**:
1. Select the **Single Note** tab
2. Choose your MIDI note (C4, C#4, etc.)
3. Set velocity (1-127) for dynamics
4. Set duration (500-5000ms) for note length + release tail
5. Click **🔴 Record Sample**

**Output**: Single WAV file like `DW6000_C4_60_vel127.wav`

### 🎹 **Range Recording**
Perfect for:
- Creating complete instrument mappings
- Batch sampling entire synthesizer patches
- Building velocity-layered instruments

**Single Velocity Range**:
- Records one sample per note across your specified range
- Fastest way to capture an entire patch
- Example: C4-C6 = 25 samples in ~12 minutes

**Multi-Velocity Range** (Professional Feature):
- Records multiple velocity layers per note
- Creates dynamically responsive instruments
- Based on musical dynamics (pianissimo to fortissimo)

### 🎯 **Velocity Layer Details**

#### **2 Layers** (Quick & Effective)
- **Soft (64)**: Mellow, filtered character  
- **Loud (127)**: Bright, full harmonics
- **Use Case**: Simple on/off dynamic behavior

#### **3 Layers** (Professional Standard)
- **Soft (48)**: Pianissimo - very gentle
- **Medium (96)**: Mezzo-forte - balanced tone
- **Loud (127)**: Fortissimo - maximum power
- **Use Case**: Realistic dynamic response

#### **4 Layers** (Full Dynamic Range)
- **Very Soft (32)**: Pianissimo - barely audible attack
- **Soft (64)**: Piano - gentle playing
- **Medium (96)**: Forte - strong but controlled  
- **Loud (127)**: Fortissimo - maximum dynamics
- **Use Case**: Expressive performance instruments

#### **Custom Layers**
- Enter your own comma-separated velocities
- Example: `40,80,120` for 3 custom layers
- **Use Case**: Hardware-specific velocity curves

### 📁 **File Organization Examples**

#### **Single Velocity**:
```
Desktop/Batcherbird Samples/DW6000/
├── DW6000_C4_60_vel127.wav
├── DW6000_C#4_61_vel127.wav
├── DW6000_D4_62_vel127.wav
└── ...
```

#### **Multi-Velocity (3 Layers)**:
```
Desktop/Batcherbird Samples/DW6000/
├── DW6000_C4_60_vel048.wav
├── DW6000_C4_60_vel096.wav  
├── DW6000_C4_60_vel127.wav
├── DW6000_C#4_61_vel048.wav
├── DW6000_C#4_61_vel096.wav
├── DW6000_C#4_61_vel127.wav
└── ...
```

### ⏱️ **Recording Time Estimates**

| Range | Velocity Layers | Total Samples | Estimated Time* |
|-------|----------------|---------------|-----------------|
| C4-C6 (25 notes) | 1 | 25 | ~12 minutes |
| C4-C6 (25 notes) | 3 | 75 | ~37 minutes |
| C2-C7 (61 notes) | 1 | 61 | ~30 minutes |
| C2-C7 (61 notes) | 4 | 244 | ~2 hours |

*Based on 2000ms duration + 200ms delay between samples*

### 🎨 **Professional Tips**

#### **Velocity Response Testing**:
1. Start with **Single Note** to test your patch's velocity response
2. Record C4 at velocities 32, 64, 96, 127
3. Listen for timbral differences (not just volume)
4. Choose appropriate velocity layer count based on character changes

#### **Duration Settings**:
- **Percussive**: 1000-2000ms (short decay)
- **Sustained**: 3000-5000ms (long pads, strings)
- **Vintage Synth**: 2000ms (good balance for most patches)

#### **Hardware Optimization**:
- Use **MIDI Panic** if notes get stuck
- Longer delays for vintage gear (some need 500ms between notes)
- Test with **Preview** before full range recording

## 🎹 Tested Hardware

| Synthesizer | Audio Interface | Status |
|-------------|----------------|--------|
| Korg DW6000 | Arturia MiniFuse | ✅ Fully Tested |
| Roland Juno-106 | Focusrite Scarlett | 🧪 Community Tested |
| Moog Subsequent 37 | Universal Audio Apollo | 🧪 Community Tested |

*Want to add your setup? [Open an issue](https://github.com/yourusername/batcherbird/issues) with your test results!*

## 🏗️ Architecture

Batcherbird uses a unique **blocking command architecture** specifically designed for professional audio on macOS:

- **Core Audio Engine**: Rust-based sampling engine with sub-millisecond timing
- **Persistent Streams**: Single audio stream per recording session (prevents WAV corruption)
- **MIDI Safety**: Automatic panic messages prevent stuck notes
- **Thread-Safe**: Dedicated audio threads with channel communication

See [Technical Documentation](TAURI_AUDIO_ARCHITECTURE.md) for detailed architecture.

## 🤝 Contributing

We welcome contributions! Areas where we need help:

- **Linux Support**: Porting the audio engine to ALSA/JACK
- **Windows Support**: DirectSound/WASAPI implementation  
- **More Hardware Testing**: Expand our compatibility matrix
- **UI/UX Improvements**: Better visual design and workflow

See [IMPLEMENTATION_PRD.md](IMPLEMENTATION_PRD.md) for technical implementation details.

## 🐛 Known Issues

- **macOS Only**: Currently supports macOS only (Linux/Windows planned for v0.3)
- **Audio Permission**: Requires microphone permission for recording
- **Export Formats**: Professional sampler format export planned for v0.2

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- Built with [Tauri](https://tauri.app/) for native performance
- Audio powered by [CPAL](https://github.com/RustAudio/cpal) and Core Audio
- MIDI handling via [midir](https://github.com/Boddlnagg/midir)
- Inspired by professional sampling workflows, built for the open-source community

---

**Professional auto-sampling shouldn't be locked behind expensive commercial tools.**  
*Batcherbird makes high-quality hardware sampling accessible to everyone.*