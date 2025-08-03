use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// Professional audio diagnostics following DAW industry standards
/// Tracks real-time performance metrics without blocking operations
#[derive(Debug)]
pub struct AudioDiagnostics {
    // Timing measurements (atomic for lock-free access)
    callback_count: AtomicU64,
    total_execution_time_ns: AtomicU64,
    max_execution_time_ns: AtomicU64,
    min_execution_time_ns: AtomicU64,
    
    // Lock contention detection
    lock_attempts: AtomicU64,
    lock_contentions: AtomicU64,
    lock_wait_time_ns: AtomicU64,
    
    // Buffer performance
    buffer_overflows: AtomicU32,
    buffer_underflows: AtomicU32,
    
    // MIDI timing accuracy
    midi_events_processed: AtomicU64,
    midi_timing_jitter_ns: AtomicU64,
    
    // Real-time safety violations
    memory_allocations: AtomicU32,
    blocking_operations: AtomicU32,
    
    // Performance thresholds (based on professional DAW standards)
    target_callback_time_ns: u64,
    warning_threshold_ns: u64,
    critical_threshold_ns: u64,
}

impl AudioDiagnostics {
    /// Create new diagnostics with professional audio thresholds
    /// 
    /// For 128 samples at 44.1kHz = ~2.9ms budget per callback
    /// Warning at 80% utilization, Critical at 95%
    pub fn new(sample_rate: u32, buffer_size: u32) -> Self {
        let callback_duration_ns = (buffer_size as f64 / sample_rate as f64 * 1_000_000_000.0) as u64;
        let warning_threshold = (callback_duration_ns as f64 * 0.8) as u64;  // 80% utilization
        let critical_threshold = (callback_duration_ns as f64 * 0.95) as u64; // 95% utilization
        
        Self {
            callback_count: AtomicU64::new(0),
            total_execution_time_ns: AtomicU64::new(0),
            max_execution_time_ns: AtomicU64::new(0),
            min_execution_time_ns: AtomicU64::new(u64::MAX),
            
            lock_attempts: AtomicU64::new(0),
            lock_contentions: AtomicU64::new(0),
            lock_wait_time_ns: AtomicU64::new(0),
            
            buffer_overflows: AtomicU32::new(0),
            buffer_underflows: AtomicU32::new(0),
            
            midi_events_processed: AtomicU64::new(0),
            midi_timing_jitter_ns: AtomicU64::new(0),
            
            memory_allocations: AtomicU32::new(0),
            blocking_operations: AtomicU32::new(0),
            
            target_callback_time_ns: callback_duration_ns,
            warning_threshold_ns: warning_threshold,
            critical_threshold_ns: critical_threshold,
        }
    }
    
    /// Start timing an audio callback (call at callback entry)
    #[inline]
    pub fn start_callback_timing(&self) -> CallbackTimer {
        CallbackTimer::new(self)
    }
    
    /// Record a lock attempt (call before any lock operation)
    #[inline]
    pub fn record_lock_attempt(&self) -> LockTimer {
        self.lock_attempts.fetch_add(1, Ordering::Relaxed);
        LockTimer::new(self)
    }
    
    /// Record buffer overflow
    #[inline]
    pub fn record_buffer_overflow(&self) {
        self.buffer_overflows.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record buffer underflow
    #[inline]
    pub fn record_buffer_underflow(&self) {
        self.buffer_underflows.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record memory allocation violation
    #[inline]
    pub fn record_memory_allocation(&self) {
        self.memory_allocations.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record blocking operation violation
    #[inline]
    pub fn record_blocking_operation(&self) {
        self.blocking_operations.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record MIDI event processing
    #[inline]
    pub fn record_midi_event(&self, timing_jitter_ns: u64) {
        self.midi_events_processed.fetch_add(1, Ordering::Relaxed);
        self.midi_timing_jitter_ns.fetch_add(timing_jitter_ns, Ordering::Relaxed);
    }
    
    /// Get comprehensive performance report
    pub fn get_performance_report(&self) -> AudioPerformanceReport {
        let callback_count = self.callback_count.load(Ordering::Relaxed);
        let total_time = self.total_execution_time_ns.load(Ordering::Relaxed);
        
        let avg_execution_time_ns = if callback_count > 0 {
            total_time / callback_count
        } else {
            0
        };
        
        let max_execution_time_ns = self.max_execution_time_ns.load(Ordering::Relaxed);
        let min_execution_time_ns = self.min_execution_time_ns.load(Ordering::Relaxed);
        
        let lock_attempts = self.lock_attempts.load(Ordering::Relaxed);
        let lock_contentions = self.lock_contentions.load(Ordering::Relaxed);
        let lock_wait_time = self.lock_wait_time_ns.load(Ordering::Relaxed);
        
        let avg_lock_wait_time_ns = if lock_attempts > 0 {
            lock_wait_time / lock_attempts
        } else {
            0
        };
        
        let midi_events = self.midi_events_processed.load(Ordering::Relaxed);
        let total_jitter = self.midi_timing_jitter_ns.load(Ordering::Relaxed);
        let avg_midi_jitter_ns = if midi_events > 0 {
            total_jitter / midi_events
        } else {
            0
        };
        
        AudioPerformanceReport {
            callback_count,
            avg_execution_time_ns,
            max_execution_time_ns,
            min_execution_time_ns: if min_execution_time_ns == u64::MAX { 0 } else { min_execution_time_ns },
            cpu_utilization_percent: (avg_execution_time_ns as f64 / self.target_callback_time_ns as f64 * 100.0) as f32,
            
            lock_attempts,
            lock_contentions,
            lock_contention_rate: if lock_attempts > 0 { lock_contentions as f32 / lock_attempts as f32 } else { 0.0 },
            avg_lock_wait_time_ns,
            
            buffer_overflows: self.buffer_overflows.load(Ordering::Relaxed),
            buffer_underflows: self.buffer_underflows.load(Ordering::Relaxed),
            
            midi_events_processed: midi_events,
            avg_midi_jitter_ns,
            
            memory_allocations: self.memory_allocations.load(Ordering::Relaxed),
            blocking_operations: self.blocking_operations.load(Ordering::Relaxed),
            
            performance_status: self.get_performance_status(avg_execution_time_ns),
        }
    }
    
    fn get_performance_status(&self, avg_execution_time_ns: u64) -> PerformanceStatus {
        if avg_execution_time_ns >= self.critical_threshold_ns {
            PerformanceStatus::Critical
        } else if avg_execution_time_ns >= self.warning_threshold_ns {
            PerformanceStatus::Warning
        } else {
            PerformanceStatus::Good
        }
    }
    
    /// Reset all diagnostics (for testing)
    pub fn reset(&self) {
        self.callback_count.store(0, Ordering::Relaxed);
        self.total_execution_time_ns.store(0, Ordering::Relaxed);
        self.max_execution_time_ns.store(0, Ordering::Relaxed);
        self.min_execution_time_ns.store(u64::MAX, Ordering::Relaxed);
        self.lock_attempts.store(0, Ordering::Relaxed);
        self.lock_contentions.store(0, Ordering::Relaxed);
        self.lock_wait_time_ns.store(0, Ordering::Relaxed);
        self.buffer_overflows.store(0, Ordering::Relaxed);
        self.buffer_underflows.store(0, Ordering::Relaxed);
        self.midi_events_processed.store(0, Ordering::Relaxed);
        self.midi_timing_jitter_ns.store(0, Ordering::Relaxed);
        self.memory_allocations.store(0, Ordering::Relaxed);
        self.blocking_operations.store(0, Ordering::Relaxed);
    }
}

/// Timer for measuring callback execution time
pub struct CallbackTimer<'a> {
    diagnostics: &'a AudioDiagnostics,
    start_time: Instant,
}

impl<'a> CallbackTimer<'a> {
    fn new(diagnostics: &'a AudioDiagnostics) -> Self {
        Self {
            diagnostics,
            start_time: Instant::now(),
        }
    }
}

impl<'a> Drop for CallbackTimer<'a> {
    fn drop(&mut self) {
        let elapsed_ns = self.start_time.elapsed().as_nanos() as u64;
        
        // Update callback count and total time
        self.diagnostics.callback_count.fetch_add(1, Ordering::Relaxed);
        self.diagnostics.total_execution_time_ns.fetch_add(elapsed_ns, Ordering::Relaxed);
        
        // Update max execution time
        let mut current_max = self.diagnostics.max_execution_time_ns.load(Ordering::Relaxed);
        while elapsed_ns > current_max {
            match self.diagnostics.max_execution_time_ns.compare_exchange_weak(
                current_max, elapsed_ns, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
        
        // Update min execution time
        let mut current_min = self.diagnostics.min_execution_time_ns.load(Ordering::Relaxed);
        while elapsed_ns < current_min {
            match self.diagnostics.min_execution_time_ns.compare_exchange_weak(
                current_min, elapsed_ns, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }
    }
}

/// Timer for measuring lock wait time
pub struct LockTimer<'a> {
    diagnostics: &'a AudioDiagnostics,
    start_time: Instant,
}

impl<'a> LockTimer<'a> {
    fn new(diagnostics: &'a AudioDiagnostics) -> Self {
        Self {
            diagnostics,
            start_time: Instant::now(),
        }
    }
    
    /// Call when lock contention is detected
    pub fn record_contention(&self) {
        self.diagnostics.lock_contentions.fetch_add(1, Ordering::Relaxed);
    }
}

impl<'a> Drop for LockTimer<'a> {
    fn drop(&mut self) {
        let elapsed_ns = self.start_time.elapsed().as_nanos() as u64;
        self.diagnostics.lock_wait_time_ns.fetch_add(elapsed_ns, Ordering::Relaxed);
    }
}

/// Comprehensive performance report
#[derive(Debug, Clone)]
pub struct AudioPerformanceReport {
    pub callback_count: u64,
    pub avg_execution_time_ns: u64,
    pub max_execution_time_ns: u64,
    pub min_execution_time_ns: u64,
    pub cpu_utilization_percent: f32,
    
    pub lock_attempts: u64,
    pub lock_contentions: u64,
    pub lock_contention_rate: f32,
    pub avg_lock_wait_time_ns: u64,
    
    pub buffer_overflows: u32,
    pub buffer_underflows: u32,
    
    pub midi_events_processed: u64,
    pub avg_midi_jitter_ns: u64,
    
    pub memory_allocations: u32,
    pub blocking_operations: u32,
    
    pub performance_status: PerformanceStatus,
}

impl AudioPerformanceReport {
    /// Check if performance is acceptable for professional audio
    pub fn is_professional_grade(&self) -> bool {
        matches!(self.performance_status, PerformanceStatus::Good) &&
        self.lock_contention_rate < 0.01 &&  // <1% lock contention
        self.memory_allocations == 0 &&      // Zero allocations in audio thread
        self.blocking_operations == 0 &&     // Zero blocking operations
        self.buffer_overflows == 0           // Zero buffer overflows
    }
    
    /// Get human-readable performance summary
    pub fn summary(&self) -> String {
        format!(
            "Performance: {:?} | CPU: {:.1}% | Contentions: {:.1}% | Violations: {} allocs, {} blocks",
            self.performance_status,
            self.cpu_utilization_percent,
            self.lock_contention_rate * 100.0,
            self.memory_allocations,
            self.blocking_operations
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceStatus {
    Good,    // <80% CPU utilization
    Warning, // 80-95% CPU utilization
    Critical, // >95% CPU utilization
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_callback_timing() {
        let diagnostics = AudioDiagnostics::new(44100, 128);
        
        {
            let _timer = diagnostics.start_callback_timing();
            thread::sleep(Duration::from_micros(100)); // Simulate 100μs callback
        }
        
        let report = diagnostics.get_performance_report();
        assert_eq!(report.callback_count, 1);
        assert!(report.avg_execution_time_ns > 50_000); // Should be >50μs
        assert!(report.avg_execution_time_ns < 500_000); // Should be <500μs
    }
    
    #[test]
    fn test_lock_contention_detection() {
        let diagnostics = AudioDiagnostics::new(44100, 128);
        
        {
            let timer = diagnostics.record_lock_attempt();
            timer.record_contention();
            thread::sleep(Duration::from_micros(50));
        }
        
        let report = diagnostics.get_performance_report();
        assert_eq!(report.lock_attempts, 1);
        assert_eq!(report.lock_contentions, 1);
        assert_eq!(report.lock_contention_rate, 1.0);
    }
    
    #[test]
    fn test_performance_status() {
        let diagnostics = AudioDiagnostics::new(44100, 128); // ~2.9ms budget
        
        // Simulate good performance (1ms execution)
        let mut good_time = 0;
        {
            let _timer = diagnostics.start_callback_timing();
            thread::sleep(Duration::from_millis(1));
            good_time = diagnostics.get_performance_report().avg_execution_time_ns;
        }
        
        assert!(good_time < diagnostics.warning_threshold_ns);
        assert_eq!(diagnostics.get_performance_status(good_time), PerformanceStatus::Good);
    }
}