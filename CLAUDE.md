# Claude Code Notes for BatcherBird

## Critical React State Management Lessons

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
- GUI: React + TypeScript + Tauri in `crates/batcherbird-gui/`
- Run development: `npm run dev` (from GUI directory)
- MIDI device selection state is managed in App.tsx and passed down to components