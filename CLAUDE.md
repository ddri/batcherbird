# Claude Code Notes for BatcherBird

## Critical React State Management Lessons


### Workflow

  1. Check PRODUCTPLAN.md for current epic/task
  2. Reference INFRA-RESEARCH.md for technical approach
  3. Implement with your checkpoint questions
  4. Update PRODUCTPLAN.md with progress

### Always Check: Props vs Local State
- **Props named "initial*" are meant to set initial values, then become local state**
- **Controlled components need BOTH `value` and `onChange` handlers**
- **Never use props directly as controlled component values without state management**

#### Bad Pattern (breaks interactivity):
```jsx
<Tabs value={initialTab}>  // Static, can't change
```

#### Good Pattern:
```jsx
const [currentTab, setCurrentTab] = useState(initialTab)
useEffect(() => setCurrentTab(initialTab), [initialTab])
<Tabs value={currentTab} onValueChange={setCurrentTab}>
```

### State Ownership Checklist
Before writing any interactive component, ask:
1. Who owns this state - parent or child?
2. Does this component need to control this value?
3. What happens when the user interacts with this?
4. Are there any "initial" props that need to become local state?

## Project Setup
- GUI: VIZIA (Rust) in `crates/batcherbird-vizia/`
- Run development: `cargo run -p batcherbird-vizia`
- CSS theme: `crates/batcherbird-vizia/src/style/theme.css` (hot-reload supported)
- Old Tauri GUI in `crates/batcherbird-gui/` is deprecated — do not modify
- App state managed in `app_data.rs` via VIZIA Model + Lens pattern



# AUDIO APPLICATION DEVELOPMENT RULES

  ## Real-Time Audio Architecture
  - NEVER use Tauri events for high-frequency data
  streaming (causes crashes)
  - ALWAYS use Tauri channels for streaming audio
  visualization data
  - FOLLOW the 3-thread pattern: UI Thread, Audio Thread,
   Visualization Thread
  - USE lock-free ring buffers (rtrb crate) between audio
   callback and visualization
  - KEEP audio thread work minimal (only peak/RMS
  calculation, never blocking operations)
  
  ## Real-Time Metering Implementation
  - IMPLEMENTED: Professional lock-free metering system
  - See REALTIME_METERS.md for complete architecture docs
  - RealtimeMeterData streams at 60fps via Tauri channels
  - Zero audio dropouts with lock-free ring buffers
  - Matches Pro Tools/Logic/Ableton standards

  ## Audio Thread Safety
  - NEVER allocate memory in audio callbacks
  - NEVER use Arc<Mutex> in audio callbacks (use
  lock-free structures)
  - ALWAYS use try_push/try_pop on ring buffers (never
  blocking variants)
  - CALCULATE visualization data in audio thread, SEND
  via separate thread

  ## Rust Audio Stack
  - USE rtrb crate for real-time ring buffers
  - USE CPAL for audio I/O (single stream for recording +
   visualization)
  - AVOID Web Audio API entirely in favor of native Rust
  processing
  - PREFER bounded channels with appropriate buffer sizes

  ## Implementation Order
  - ALWAYS implement backend streaming first, then
  frontend consumption
  - START with simple peak/RMS data before complex
  waveform visualization
  - TEST ring buffer performance separately before
  integrating with Tauri
  - VERIFY no audio dropouts before adding UI features