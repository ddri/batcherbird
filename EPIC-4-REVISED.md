# Epic 4 Revised Plan - Professional Sampling Features

## Market Research Summary

Based on extensive research of SampleRobot ($249) and Samplit (discontinued), we've identified critical user pain points:

### What Users Dislike About Existing Tools
- **SampleRobot**: "unsufferable bug bag", "hair-pulling moments", crashes, loop export bugs
- **Samplit**: No longer developed, skips notes, buggy auto-looping, stability issues
- **Both**: Overly complex features that don't work reliably

### What Users Actually Want
1. **Reliability over features** - "I had nothing but problems"
2. **Great auto-looping** - "godsend feature" when it works
3. **Simple velocity layers** - 2-4 layers max, not 16
4. **Basic note ranges** - 1-4 octaves for synths, not full 88-key pianos
5. **Open formats** - DecentSampler and SFZ have huge communities

## Revised Epic 4 Scope (6 weeks, down from 12)

### 4.1: Smart Velocity Layers & Auto-Loop (2 weeks) ✨ CRITICAL
**What we're building:**
- 2, 3, or 4 velocity layers with smart defaults
- Industry-leading FFT-based auto-loop detection
- Visual loop editor with waveform display
- Loop presets for different synth types

**What we're NOT building:**
- 8, 16 layer support
- Complex velocity curves
- Custom velocity mapping UI

### 4.2: Flexible Note Range Recording (2 weeks)
**What we're building:**
- User selects octave range (1-4 octaves)
- Two modes: "Every Note" or "Every 3rd Note"
- Clear time estimates and progress
- Pause/resume for long sessions

**What we're NOT building:**
- Full 88-key piano sampling
- Complex interval patterns
- Automatic transposition

### 4.3: Simple Templates & Batch (1 week)
**What we're building:**
- Presets: Pad, Lead, Bass, Percussion
- Batch process existing samples
- Save custom presets

**What we're NOT building:**
- Complex project management
- Multi-user collaboration
- Version control

### 4.4: Open Format Export (1 week)
**What we're building:**
- Perfect DecentSampler export (.dslibrary)
- Complete SFZ 2.0 export
- All metadata preserved

**What we're NOT building:**
- Kontakt (licensing required)
- EXS24, HALion (proprietary)
- SoundFont (obsolete)

## Technical Advantages

### Why We'll Succeed Where Others Failed

1. **Lock-Free Architecture** (from INFRA-RESEARCH.md)
   - No crashes at high frequency
   - Professional 3-thread model
   - Already proven in Epic 1-3

2. **FFT-Based Loop Detection** (from Epic 3)
   - 5-10x faster than competitors
   - Already implemented and tested
   - Just needs UI integration

3. **Simpler is Better**
   - Do 4 things perfectly vs 20 things poorly
   - Focus on what users actually use
   - Ship updates frequently

## Success Metrics

### What "Done" Looks Like
- ✅ Zero crashes in 4+ hour sessions
- ✅ Auto-loop works 95%+ of the time
- ✅ Velocity layers record reliably
- ✅ DecentSampler export opens perfectly
- ✅ Users say "it just works"

### What We're NOT Measuring
- ❌ Feature count vs competitors
- ❌ Support for every format
- ❌ Complex enterprise features

## Pricing Strategy

**Target: $49-79** (vs SampleRobot at $249)
- Undercut on price
- Overdeliver on reliability
- Open source advantage

## Development Timeline

```
Week 1-2: Auto-loop detection + Velocity layers
Week 3-4: Note range recording
Week 5: Templates + Batch processing
Week 6: DecentSampler + SFZ export
```

## Key Insight

**"Users will forgive missing features. They won't forgive crashes and bugs."**

BatcherBird will be the sampling tool that just works. Every time.