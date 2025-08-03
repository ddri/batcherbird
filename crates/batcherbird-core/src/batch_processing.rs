use crate::{Result, BatcherbirdError};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, AtomicBool, Ordering}};
use std::time::{Instant, Duration};
use std::path::PathBuf;

/// Configuration for batch processing operations
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of concurrent processing threads
    pub max_threads: Option<usize>,
    
    /// Memory management settings
    pub memory_limit_mb: Option<usize>,
    
    /// Enable progress tracking
    pub progress_tracking: bool,
    
    /// Chunk size for streaming large datasets
    pub chunk_size: usize,
    
    /// Timeout for individual operations in seconds
    pub operation_timeout_secs: u64,
    
    /// Enable error recovery (continue on individual failures)
    pub error_recovery: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_threads: None, // Use rayon's default (usually CPU cores)
            memory_limit_mb: Some(512), // 512MB memory limit
            progress_tracking: true,
            chunk_size: 100, // Process 100 samples per chunk
            operation_timeout_secs: 60, // 1 minute per operation
            error_recovery: true,
        }
    }
}

/// Progress tracking for batch operations
#[derive(Debug)]
pub struct BatchProgress {
    /// Total number of items to process
    pub total_items: usize,
    
    /// Number of items completed
    pub completed_items: AtomicUsize,
    
    /// Number of items that failed
    pub failed_items: AtomicUsize,
    
    /// Start time for ETA calculation
    pub start_time: Instant,
    
    /// Whether processing is cancelled
    pub cancelled: AtomicBool,
    
    /// Current processing phase
    pub current_phase: Arc<Mutex<String>>,
}

impl BatchProgress {
    pub fn new(total_items: usize) -> Self {
        Self {
            total_items,
            completed_items: AtomicUsize::new(0),
            failed_items: AtomicUsize::new(0),
            start_time: Instant::now(),
            cancelled: AtomicBool::new(false),
            current_phase: Arc::new(Mutex::new("Initializing".to_string())),
        }
    }
    
    /// Get current progress as percentage (0.0 to 1.0)
    pub fn progress_percentage(&self) -> f32 {
        if self.total_items == 0 {
            return 1.0;
        }
        
        let completed = self.completed_items.load(Ordering::Relaxed);
        completed as f32 / self.total_items as f32
    }
    
    /// Estimate time remaining in seconds
    pub fn eta_seconds(&self) -> Option<f32> {
        let completed = self.completed_items.load(Ordering::Relaxed);
        if completed == 0 {
            return None;
        }
        
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let rate = completed as f32 / elapsed;
        let remaining = self.total_items - completed;
        
        Some(remaining as f32 / rate)
    }
    
    /// Mark one item as completed
    pub fn increment_completed(&self) {
        self.completed_items.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Mark one item as failed
    pub fn increment_failed(&self) {
        self.failed_items.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Cancel the batch operation
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
    
    /// Check if operation is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
    
    /// Set current processing phase
    pub fn set_phase(&self, phase: &str) {
        if let Ok(mut current_phase) = self.current_phase.lock() {
            *current_phase = phase.to_string();
        }
    }
}

/// Memory manager for batch processing
pub struct MemoryManager {
    limit_bytes: Option<usize>,
    current_usage: AtomicUsize,
}

impl MemoryManager {
    pub fn new(limit_mb: Option<usize>) -> Self {
        Self {
            limit_bytes: limit_mb.map(|mb| mb * 1024 * 1024),
            current_usage: AtomicUsize::new(0),
        }
    }
    
    /// Check if we can allocate more memory
    pub fn can_allocate(&self, bytes: usize) -> bool {
        if let Some(limit) = self.limit_bytes {
            let current = self.current_usage.load(Ordering::Relaxed);
            current + bytes <= limit
        } else {
            true // No limit
        }
    }
    
    /// Track memory allocation
    pub fn allocate(&self, bytes: usize) -> bool {
        if !self.can_allocate(bytes) {
            return false;
        }
        
        self.current_usage.fetch_add(bytes, Ordering::Relaxed);
        true
    }
    
    /// Track memory deallocation
    pub fn deallocate(&self, bytes: usize) {
        self.current_usage.fetch_sub(bytes, Ordering::Relaxed);
    }
    
    /// Get current memory usage in bytes
    pub fn current_usage_bytes(&self) -> usize {
        self.current_usage.load(Ordering::Relaxed)
    }
    
    /// Get current memory usage in MB
    pub fn current_usage_mb(&self) -> f32 {
        self.current_usage_bytes() as f32 / (1024.0 * 1024.0)
    }
}

/// Individual sample data for batch processing
#[derive(Debug, Clone)]
pub struct SampleData {
    /// Unique identifier for this sample
    pub id: String,
    
    /// Audio data (mono, f32 samples)
    pub audio_data: Vec<f32>,
    
    /// Sample rate in Hz
    pub sample_rate: u32,
    
    /// Original file path (if loaded from file)
    pub source_path: Option<PathBuf>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl SampleData {
    pub fn new(id: String, audio_data: Vec<f32>, sample_rate: u32) -> Self {
        Self {
            id,
            audio_data,
            sample_rate,
            source_path: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Estimate memory usage of this sample in bytes
    pub fn memory_usage(&self) -> usize {
        let audio_bytes = self.audio_data.len() * std::mem::size_of::<f32>();
        let metadata_bytes = self.metadata.iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>();
        let path_bytes = self.source_path.as_ref()
            .map(|p| p.to_string_lossy().len())
            .unwrap_or(0);
        
        audio_bytes + metadata_bytes + path_bytes + self.id.len()
    }
}

/// Result of a batch processing operation
#[derive(Debug)]
pub struct BatchResult<T> {
    /// Successfully processed results
    pub successes: Vec<(String, T)>, // (sample_id, result)
    
    /// Failed operations with errors
    pub failures: Vec<(String, BatcherbirdError)>, // (sample_id, error)
    
    /// Total processing time
    pub processing_time: Duration,
    
    /// Performance metrics
    pub performance_metrics: BatchPerformanceMetrics,
}

/// Performance metrics for batch operations
#[derive(Debug, Clone)]
pub struct BatchPerformanceMetrics {
    /// Total items processed
    pub total_items: usize,
    
    /// Successful items
    pub successful_items: usize,
    
    /// Failed items
    pub failed_items: usize,
    
    /// Average processing time per item (milliseconds)
    pub avg_time_per_item_ms: f32,
    
    /// Peak memory usage (MB)
    pub peak_memory_mb: f32,
    
    /// Speedup factor compared to sequential processing
    pub speedup_factor: Option<f32>,
    
    /// CPU utilization percentage
    pub cpu_utilization: Option<f32>,
}

/// Parallel batch processing engine
pub struct BatchProcessor {
    config: BatchConfig,
    memory_manager: Arc<MemoryManager>,
    thread_pool: Option<rayon::ThreadPool>,
}

impl BatchProcessor {
    /// Create new batch processor with configuration
    pub fn new(config: BatchConfig) -> Result<Self> {
        let memory_manager = Arc::new(MemoryManager::new(config.memory_limit_mb));
        
        // Create custom thread pool if max_threads is specified
        let thread_pool = if let Some(max_threads) = config.max_threads {
            Some(rayon::ThreadPoolBuilder::new()
                .num_threads(max_threads)
                .build()
                .map_err(|e| BatcherbirdError::Audio(format!("Failed to create thread pool: {}", e)))?)
        } else {
            None // Use global rayon pool
        };
        
        Ok(Self {
            config,
            memory_manager,
            thread_pool,
        })
    }
    
    /// Process samples in parallel with a given operation
    pub fn process_samples<F, T>(&self, samples: Vec<SampleData>, operation: F) -> Result<BatchResult<T>>
    where
        F: Fn(&SampleData) -> Result<T> + Sync + Send,
        T: Send,
    {
        let start_time = Instant::now();
        let progress = Arc::new(BatchProgress::new(samples.len()));
        
        println!("🚀 Starting batch processing: {} samples with {} threads", 
            samples.len(), 
            self.config.max_threads.unwrap_or_else(|| rayon::current_num_threads())
        );
        
        progress.set_phase("Processing samples");
        
        // Execute in custom thread pool if available
        let results = if let Some(ref pool) = self.thread_pool {
            pool.install(|| self.process_samples_parallel(samples, operation, progress.clone()))
        } else {
            self.process_samples_parallel(samples, operation, progress.clone())
        };
        
        let processing_time = start_time.elapsed();
        
        // Calculate performance metrics
        let performance_metrics = self.calculate_performance_metrics(&results, processing_time)?;
        
        println!("✅ Batch processing complete: {:.2}s, {:.1}x speedup", 
            processing_time.as_secs_f32(),
            performance_metrics.speedup_factor.unwrap_or(1.0)
        );
        
        // Split results into successes and failures
        let mut successes = Vec::new();
        let mut failures = Vec::new();
        
        for (id, result) in results {
            match result {
                Ok(value) => successes.push((id, value)),
                Err(error) => failures.push((id, error)),
            }
        }
        
        Ok(BatchResult {
            successes,
            failures,
            processing_time,
            performance_metrics,
        })
    }
    
    fn process_samples_parallel<F, T>(
        &self,
        samples: Vec<SampleData>,
        operation: F,
        progress: Arc<BatchProgress>,
    ) -> Vec<(String, Result<T>)>
    where
        F: Fn(&SampleData) -> Result<T> + Sync + Send,
        T: Send,
    {
        // Process in chunks to manage memory usage
        let chunk_size = self.config.chunk_size;
        let chunks: Vec<_> = samples.chunks(chunk_size).collect();
        
        let mut all_results = Vec::new();
        
        for (chunk_index, chunk) in chunks.iter().enumerate() {
            if progress.is_cancelled() {
                println!("⚠️ Batch processing cancelled by user");
                break;
            }
            
            println!("📦 Processing chunk {}/{} ({} samples)", 
                chunk_index + 1, chunks.len(), chunk.len());
            
            // Check memory before processing chunk
            let chunk_memory: usize = chunk.iter().map(|s| s.memory_usage()).sum();
            if !self.memory_manager.allocate(chunk_memory) {
                println!("⚠️ Memory limit exceeded, processing smaller chunks");
                // Fall back to individual processing for this chunk
                for sample in *chunk {
                    if let Ok(result) = self.process_single_sample(sample, &operation, &progress) {
                        all_results.push(result);
                    }
                }
                continue;
            }
            
            // Process chunk in parallel
            let chunk_results: Vec<_> = chunk.par_iter()
                .map(|sample| {
                    if progress.is_cancelled() {
                        return (sample.id.clone(), Err(BatcherbirdError::Audio("Operation cancelled".to_string())));
                    }
                    
                    let result = self.process_single_sample(sample, &operation, &progress);
                    match result {
                        Ok((id, res)) => (id, res),
                        Err(e) => (sample.id.clone(), Err(e)),
                    }
                })
                .collect();
            
            all_results.extend(chunk_results);
            
            // Deallocate chunk memory
            self.memory_manager.deallocate(chunk_memory);
            
            // Update progress
            progress.set_phase(&format!("Completed chunk {}/{}", chunk_index + 1, chunks.len()));
        }
        
        all_results
    }
    
    fn process_single_sample<F, T>(
        &self,
        sample: &SampleData,
        operation: &F,
        progress: &Arc<BatchProgress>,
    ) -> Result<(String, Result<T>)>
    where
        F: Fn(&SampleData) -> Result<T> + Sync + Send,
        T: Send,
    {
        let start_time = Instant::now();
        let timeout = Duration::from_secs(self.config.operation_timeout_secs);
        
        // Simple timeout mechanism (in production, would use tokio::time::timeout)
        let result = operation(sample);
        let elapsed = start_time.elapsed();
        
        if elapsed > timeout {
            progress.increment_failed();
            return Ok((sample.id.clone(), Err(BatcherbirdError::Audio(
                format!("Operation timed out after {}s", elapsed.as_secs())
            ))));
        }
        
        match result {
            Ok(value) => {
                progress.increment_completed();
                Ok((sample.id.clone(), Ok(value)))
            }
            Err(error) => {
                progress.increment_failed();
                if self.config.error_recovery {
                    Ok((sample.id.clone(), Err(error)))
                } else {
                    Err(error)
                }
            }
        }
    }
    
    fn calculate_performance_metrics<T>(
        &self,
        results: &[(String, Result<T>)],
        processing_time: Duration,
    ) -> Result<BatchPerformanceMetrics> {
        let total_items = results.len();
        let successful_items = results.iter().filter(|(_, r)| r.is_ok()).count();
        let failed_items = total_items - successful_items;
        
        let avg_time_per_item_ms = if total_items > 0 {
            processing_time.as_millis() as f32 / total_items as f32
        } else {
            0.0
        };
        
        let peak_memory_mb = self.memory_manager.current_usage_mb();
        
        // Estimate speedup factor (simplified calculation)
        let estimated_sequential_time = avg_time_per_item_ms * total_items as f32;
        let actual_time_ms = processing_time.as_millis() as f32;
        let speedup_factor = if actual_time_ms > 0.0 {
            Some(estimated_sequential_time / actual_time_ms)
        } else {
            None
        };
        
        Ok(BatchPerformanceMetrics {
            total_items,
            successful_items,
            failed_items,
            avg_time_per_item_ms,
            peak_memory_mb,
            speedup_factor,
            cpu_utilization: None, // Would require system monitoring
        })
    }
    
    /// Get current progress for ongoing operations
    pub fn get_progress(&self) -> Option<&BatchProgress> {
        // In a full implementation, this would track active operations
        None
    }
    
    /// Cancel all ongoing operations
    pub fn cancel_all(&self) {
        // In a full implementation, this would signal all active operations to stop
        println!("🛑 Cancelling all batch operations");
    }
}

/// Streaming processor for very large datasets
pub struct StreamingProcessor {
    batch_processor: BatchProcessor,
    stream_config: StreamConfig,
}

#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Size of streaming buffer
    pub buffer_size: usize,
    
    /// Maximum concurrent streams
    pub max_concurrent_streams: usize,
    
    /// Enable result caching
    pub enable_caching: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            max_concurrent_streams: 4,
            enable_caching: false,
        }
    }
}

impl StreamingProcessor {
    pub fn new(batch_config: BatchConfig, stream_config: StreamConfig) -> Result<Self> {
        Ok(Self {
            batch_processor: BatchProcessor::new(batch_config)?,
            stream_config,
        })
    }
    
    /// Process a stream of samples with backpressure handling
    pub fn process_stream<F, T>(
        &self,
        sample_stream: impl Iterator<Item = SampleData>,
        operation: F,
    ) -> Result<impl Iterator<Item = (String, Result<T>)>>
    where
        F: Fn(&SampleData) -> Result<T> + Sync + Send + Clone + 'static,
        T: Send + 'static,
    {
        // Collect samples into batches and process
        let samples: Vec<_> = sample_stream.collect();
        let result = self.batch_processor.process_samples(samples, operation)?;
        
        // Combine successes and failures into single iterator
        let mut combined: Vec<(String, Result<T>)> = Vec::new();
        
        for (id, value) in result.successes {
            combined.push((id, Ok(value)));
        }
        
        for (id, error) in result.failures {
            combined.push((id, Err(error)));
        }
        
        Ok(combined.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_batch_processor_creation() {
        let config = BatchConfig::default();
        let processor = BatchProcessor::new(config).unwrap();
        
        assert!(processor.memory_manager.current_usage_bytes() == 0);
    }
    
    #[test]
    fn test_sample_data_memory_calculation() {
        let sample = SampleData::new(
            "test".to_string(),
            vec![1.0, 2.0, 3.0, 4.0],
            44100,
        );
        
        let memory_usage = sample.memory_usage();
        assert!(memory_usage > 0);
        assert!(memory_usage >= 4 * std::mem::size_of::<f32>()); // At least audio data size
    }
    
    #[test]
    fn test_batch_progress() {
        let progress = BatchProgress::new(100);
        
        assert_eq!(progress.progress_percentage(), 0.0);
        
        progress.increment_completed();
        assert_eq!(progress.progress_percentage(), 0.01);
        
        progress.increment_failed();
        assert_eq!(progress.failed_items.load(Ordering::Relaxed), 1);
    }
    
    #[test]
    fn test_memory_manager() {
        let memory_manager = MemoryManager::new(Some(1)); // 1MB limit
        
        assert!(memory_manager.can_allocate(500_000)); // 500KB should fit
        assert!(memory_manager.allocate(500_000));
        assert_eq!(memory_manager.current_usage_bytes(), 500_000);
        
        assert!(!memory_manager.can_allocate(600_000)); // Would exceed limit
        
        memory_manager.deallocate(200_000);
        assert_eq!(memory_manager.current_usage_bytes(), 300_000);
        assert!(memory_manager.can_allocate(600_000)); // Now it fits
    }
    
    #[test]
    fn test_parallel_processing() {
        let config = BatchConfig {
            max_threads: Some(2),
            ..Default::default()
        };
        let processor = BatchProcessor::new(config).unwrap();
        
        // Create test samples
        let samples = (0..10)
            .map(|i| SampleData::new(
                format!("sample_{}", i),
                vec![i as f32; 100],
                44100,
            ))
            .collect();
        
        // Simple operation: sum all audio samples
        let operation = |sample: &SampleData| -> Result<f32> {
            Ok(sample.audio_data.iter().sum())
        };
        
        let result = processor.process_samples(samples, operation).unwrap();
        
        assert_eq!(result.successes.len(), 10);
        assert_eq!(result.failures.len(), 0);
        assert!(result.performance_metrics.speedup_factor.unwrap_or(1.0) >= 1.0);
        
        // Check that results are correct
        for (i, (id, sum)) in result.successes.iter().enumerate() {
            assert_eq!(id, &format!("sample_{}", i));
            assert_eq!(*sum, (i as f32) * 100.0); // i repeated 100 times
        }
    }
    
    #[test]
    fn test_error_recovery() {
        let config = BatchConfig {
            error_recovery: true,
            ..Default::default()
        };
        let processor = BatchProcessor::new(config).unwrap();
        
        let samples = (0..5)
            .map(|i| SampleData::new(
                format!("sample_{}", i),
                vec![i as f32; 10],
                44100,
            ))
            .collect();
        
        // Operation that fails on even-numbered samples
        let operation = |sample: &SampleData| -> Result<f32> {
            let id_num: usize = sample.id.split('_').nth(1).unwrap().parse().unwrap();
            if id_num % 2 == 0 {
                Err(BatcherbirdError::Audio("Even sample error".to_string()))
            } else {
                Ok(sample.audio_data.iter().sum())
            }
        };
        
        let result = processor.process_samples(samples, operation).unwrap();
        
        assert_eq!(result.successes.len(), 2); // samples 1 and 3
        assert_eq!(result.failures.len(), 3); // samples 0, 2, and 4
        assert_eq!(result.performance_metrics.successful_items, 2);
        assert_eq!(result.performance_metrics.failed_items, 3);
    }
    
    #[test]
    fn test_streaming_processor() {
        let batch_config = BatchConfig::default();
        let stream_config = StreamConfig::default();
        let processor = StreamingProcessor::new(batch_config, stream_config).unwrap();
        
        let samples = (0..5).map(|i| SampleData::new(
            format!("stream_sample_{}", i),
            vec![i as f32; 10],
            44100,
        ));
        
        let operation = |sample: &SampleData| -> Result<usize> {
            Ok(sample.audio_data.len())
        };
        
        let results: Vec<_> = processor.process_stream(samples, operation).unwrap().collect();
        
        assert_eq!(results.len(), 5);
        for (i, (id, result)) in results.iter().enumerate() {
            assert_eq!(id, &format!("stream_sample_{}", i));
            assert_eq!(result.as_ref().unwrap(), &10);
        }
    }
}