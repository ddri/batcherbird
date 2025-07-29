# The Heartbeat Pattern: What We Learned About Audio Threading from Professional DAWs

When building audio software, one of the most challenging aspects is managing the relationship between user interface actions and real-time audio processing. Recently, while developing Batcherbird—a tool for sampling hardware synthesizers—we encountered a fundamental architectural challenge that led us to study how professional Digital Audio Workstations handle audio threading. What we discovered changed our entire approach to audio playback.

## The Initial Problem

Our first implementation seemed straightforward enough. When a user pressed play, we would create an audio stream, start it playing, and when they pressed stop, we would destroy the stream. This mirrors how many developers naturally think about resource management: create resources when needed, clean them up when done.

The Rust compiler, however, had other ideas. When we tried to store our audio stream in a global state that could be accessed from our user interface thread, we hit a wall of compiler errors about `Send` and `Sync` traits. The audio stream, it turned out, could not be safely shared between threads on macOS—and likely other platforms as well.

This forced us to step back and reconsider our entire approach. Rather than fighting the compiler, we decided to understand why these constraints existed and how professional audio software solves this problem.

## Understanding the Constraints

Audio programming operates under unique constraints that don't exist in most software domains. The audio callback—the function that fills the buffer with audio samples—runs in a high-priority thread that must meet strict timing deadlines. On a typical system running at 44.1kHz with a 256-sample buffer, the audio callback must complete its work every 5.8 milliseconds. Missing this deadline results in audible glitches, pops, or dropouts.

This real-time constraint means the audio callback cannot perform any operation that might block or take an unpredictable amount of time. No memory allocation, no mutex locks, no file I/O—nothing that could cause the thread to miss its deadline.

Platform-specific audio APIs like Core Audio on macOS enforce these constraints at the type system level. The stream objects they provide are deliberately not thread-safe, preventing developers from accidentally sharing them across threads where synchronization might introduce latency.

## The Professional Approach

To understand how to work within these constraints, we studied how professional DAWs like Ableton Live handle audio playback. What we found was elegantly simple: they never stop the audio stream.

In Ableton, pressing play or stop doesn't create or destroy audio streams. Instead, the audio engine runs continuously from the moment the application starts until it closes. The stream is like a heartbeat—always pumping, always ready. When playback is stopped, the audio callback simply outputs silence instead of audio samples.

This approach offers several critical advantages. First, there's zero latency when starting playback because the stream is already running. Second, it eliminates any glitches that might occur from stream creation or destruction. Third, it provides a consistent timing reference, as the audio clock never stops ticking.

## The Architecture Pattern

The pattern we discovered separates concerns into distinct layers. At the lowest level, an audio thread runs continuously with the stream, never blocking, never stopping. This thread communicates with the rest of the application through lock-free atomic operations—simple flags and counters that can be read and written without synchronization.

Above this sits the application logic layer, which manages what should be played and when. This layer loads audio files, manages playback position, and responds to user input. It communicates with the audio thread by updating atomic flags and filling buffers that the audio thread consumes.

The user interface layer sits at the top, responding to user actions by updating the application state. When a user presses play, the UI doesn't touch the audio stream—it simply sets an atomic flag that the audio thread will see on its next callback.

## Implementation in Batcherbird

Looking at our existing codebase, we realized we had actually implemented this pattern correctly for audio monitoring. Our `SamplingEngine` creates a monitoring stream that runs continuously, using atomic flags to control when it should process input. We had simply failed to recognize that playback should follow the same pattern.

The corrected implementation creates a dedicated thread for audio playback when the playback engine initializes. This thread creates the audio stream and keeps it running indefinitely. The audio callback checks an atomic flag to determine whether to output audio samples or silence. Playback control becomes a matter of flipping atomic flags rather than managing stream lifecycle.

This approach also elegantly solves our thread-safety issues. Since the stream lives entirely within its dedicated thread and never needs to be accessed from other threads, the compiler's `Send` and `Sync` constraints are satisfied.

## Lessons Learned

This experience reinforced several important lessons about system design. First, when a compiler or framework imposes seemingly restrictive constraints, it's often guiding you toward a better architecture. Our initial frustration with Rust's type system led us to discover a more robust design.

Second, studying how professionals solve problems in your domain is invaluable. The audio threading pattern used by DAWs has evolved over decades to meet the demanding requirements of professional audio production. There's wisdom encoded in these architectural patterns.

Finally, consistency in architecture matters. We had already implemented the correct pattern for audio monitoring but failed to recognize that playback should follow the same approach. Now, both systems use the same threading model, making the codebase more coherent and maintainable.

## Conclusion

The heartbeat pattern—keeping audio streams running continuously—might seem counterintuitive at first. Our instinct as programmers is often to create resources only when needed. But in the real-time world of audio processing, the cost of starting and stopping is too high. Instead, like a heartbeat that never stops, the audio stream runs continuously, ready to pump audio or silence as needed.

This architectural pattern, refined over decades by professional audio software, provides the foundation for glitch-free, low-latency audio playback. By understanding and implementing this pattern in Batcherbird, we've built our audio playback on the same solid foundation used by the tools professionals rely on every day.

The next time you press play in your favorite DAW, remember: you're not starting the audio engine—you're just telling an already-beating heart what kind of blood to pump.