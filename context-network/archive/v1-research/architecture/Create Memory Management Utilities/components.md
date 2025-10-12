# Component Specifications - Memory Management Utilities

## 1. Connection Pool Component

### Interface Design

```rust
/// Manages connection lifecycle for a specific connection type
#[async_trait]
pub trait ConnectionManager: Send + Sync + 'static {
    /// The connection type managed by this manager
    type Connection: Send + 'static;
    
    /// Connection-specific errors
    type Error: Into<PatinoxError> + Send + Sync + 'static;
    
    /// Create a new connection
    async fn create(&self) -> Result<Self::Connection, Self::Error>;
    
    /// Check if a connection is still valid
    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error>;
    
    /// Recycle a connection (optional cleanup/reset)
    async fn recycle(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn) // Default implementation is no-op
    }
    
    /// Get connection timeout for this manager
    fn connect_timeout(&self) -> Duration {
        Duration::from_secs(30) // Default timeout
    }
}

/// Configuration for connection pool behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Minimum number of connections to maintain
    pub min_size: usize,
    
    /// Maximum number of connections allowed
    pub max_size: usize,
    
    /// Timeout for acquiring a connection from the pool
    pub acquire_timeout: Duration,
    
    /// Time before idle connections are closed
    pub idle_timeout: Option<Duration>,
    
    /// Interval for health checking idle connections
    pub health_check_interval: Duration,
    
    /// Enable fair queuing for connection requests
    pub fair_queue: bool,
}

/// Main connection pool implementation
pub struct Pool<M: ConnectionManager> {
    manager: M,
    config: PoolConfig,
    inner: Arc<PoolInner<M>>,
    metrics: Arc<PoolMetrics>,
    _cleanup_task: JoinHandle<()>,
}

/// RAII guard for pooled connections
pub struct PooledConnection<M: ConnectionManager> {
    connection: Option<M::Connection>,
    pool: Weak<PoolInner<M>>,
    metrics: Arc<PoolMetrics>,
    acquired_at: Instant,
}
```

### Implementation Details

**Pool Internal Structure**:
```rust
struct PoolInner<M: ConnectionManager> {
    available: Arc<Mutex<VecDeque<IdleConnection<M::Connection>>>>,
    active_count: AtomicUsize,
    waiting: Arc<Mutex<VecDeque<Sender<Result<PooledConnection<M>, PoolError>>>>>,
    shutdown: AtomicBool,
}

struct IdleConnection<T> {
    connection: T,
    idle_since: Instant,
    health_checked_at: Instant,
}
```

**Key Algorithms**:
1. **Fair Scheduling**: FIFO queue for waiting requests
2. **Health Checking**: Periodic validation of idle connections
3. **Graceful Shutdown**: Wait for active connections to complete
4. **Metrics Collection**: Track acquisition time, pool utilization

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    #[error("Pool is at capacity and timed out waiting for connection")]
    Timeout,
    
    #[error("Pool is shutting down")]
    ShuttingDown,
    
    #[error("Connection manager error: {0}")]
    Manager(Box<dyn std::error::Error + Send + Sync>),
    
    #[error("Connection validation failed: {0}")]
    ValidationFailed(String),
}

impl PoolError {
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            PoolError::Timeout => RecoveryStrategy::Retry,
            PoolError::ShuttingDown => RecoveryStrategy::Fail,
            PoolError::Manager(_) => RecoveryStrategy::Fallback,
            PoolError::ValidationFailed(_) => RecoveryStrategy::Retry,
        }
    }
}
```

### Performance Specifications

- **Acquisition Time**: < 1ms for warm pool (95th percentile)
- **Throughput**: 10,000+ acquisitions/second on modern hardware
- **Memory Overhead**: ~100 bytes per pooled connection
- **Fair Scheduling**: No request starvation under normal load

## 2. Resource Management Component

### Interface Design

```rust
/// RAII guard for async resource cleanup
pub struct AsyncResourceGuard<T> {
    resource: Option<T>,
    cleanup: Option<CleanupFn<T>>,
    registry_id: ResourceId,
    registry: Weak<ResourceRegistry>,
}

/// Async cleanup function type
type CleanupFn<T> = Box<dyn FnOnce(T) -> BoxFuture<'static, Result<(), CleanupError>> + Send>;

/// Central registry for resource tracking
pub struct ResourceRegistry {
    active: Arc<RwLock<HashMap<ResourceId, ResourceInfo>>>,
    cleanup_tx: UnboundedSender<CleanupRequest>,
    monitor: Arc<dyn Monitor>,
    shutdown: Arc<AtomicBool>,
}

/// Information tracked for each resource
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub type_name: &'static str,
    pub created_at: Instant,
    pub size_bytes: Option<usize>,
    pub metadata: HashMap<String, String>,
}

/// Resource cleanup request
pub struct CleanupRequest {
    pub resource_id: ResourceId,
    pub cleanup: BoxFuture<'static, Result<(), CleanupError>>,
    pub priority: CleanupPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CleanupPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}
```

### Resource Guard Implementation

```rust
impl<T> AsyncResourceGuard<T> {
    /// Create a new resource guard with cleanup function
    pub fn new<F, Fut>(resource: T, cleanup: F) -> Self 
    where
        F: FnOnce(T) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), CleanupError>> + Send + 'static,
    {
        let registry = GLOBAL_REGISTRY.clone();
        let registry_id = ResourceId::generate();
        
        // Register resource
        let info = ResourceInfo {
            type_name: std::any::type_name::<T>(),
            created_at: Instant::now(),
            size_bytes: std::mem::size_of::<T>().into(),
            metadata: HashMap::new(),
        };
        
        registry.register(registry_id, info);
        
        Self {
            resource: Some(resource),
            cleanup: Some(Box::new(move |res| Box::pin(cleanup(res)))),
            registry_id,
            registry: Arc::downgrade(&registry),
        }
    }
    
    /// Get immutable reference to resource
    pub fn get(&self) -> &T {
        self.resource.as_ref().expect("Resource was consumed")
    }
    
    /// Get mutable reference to resource  
    pub fn get_mut(&mut self) -> &mut T {
        self.resource.as_mut().expect("Resource was consumed")
    }
    
    /// Consume the guard and return the resource
    pub fn into_inner(mut self) -> T {
        self.resource.take().expect("Resource was consumed")
    }
    
    /// Manually trigger cleanup (consumes the guard)
    pub async fn cleanup(mut self) -> Result<(), CleanupError> {
        if let (Some(resource), Some(cleanup)) = (self.resource.take(), self.cleanup.take()) {
            cleanup(resource).await
        } else {
            Ok(())
        }
    }
}

impl<T> Drop for AsyncResourceGuard<T> {
    fn drop(&mut self) {
        if let (Some(resource), Some(cleanup)) = (self.resource.take(), self.cleanup.take()) {
            // Send cleanup request to background task
            let request = CleanupRequest {
                resource_id: self.registry_id,
                cleanup: cleanup(resource),
                priority: CleanupPriority::Normal,
            };
            
            if let Some(registry) = self.registry.upgrade() {
                registry.schedule_cleanup(request);
            }
        }
    }
}
```

### Resource Registry Implementation

```rust
impl ResourceRegistry {
    pub fn new(monitor: Arc<dyn Monitor>) -> Self {
        let (cleanup_tx, cleanup_rx) = unbounded_channel();
        
        let registry = Self {
            active: Arc::new(RwLock::new(HashMap::new())),
            cleanup_tx,
            monitor,
            shutdown: Arc::new(AtomicBool::new(false)),
        };
        
        // Start cleanup task
        registry.start_cleanup_task(cleanup_rx);
        
        registry
    }
    
    pub fn register(&self, id: ResourceId, info: ResourceInfo) {
        let mut active = self.active.blocking_write();
        active.insert(id, info);
        
        // Record metric
        self.monitor.record_event(MonitorEvent {
            event_type: MonitorEventType::ResourceCreated,
            data: json!({ "resource_id": id, "type": info.type_name }),
        });
    }
    
    pub async fn force_cleanup_all(&self) -> Result<usize, CleanupError> {
        let active = self.active.read().await;
        let count = active.len();
        
        // Send high-priority cleanup for all resources
        for (id, _) in active.iter() {
            // Force cleanup implementation
        }
        
        Ok(count)
    }
    
    fn start_cleanup_task(&self, mut rx: UnboundedReceiver<CleanupRequest>) {
        let active = Arc::clone(&self.active);
        let monitor = Arc::clone(&self.monitor);
        let shutdown = Arc::clone(&self.shutdown);
        
        tokio::spawn(async move {
            let mut pending = BinaryHeap::new(); // Priority queue
            
            while !shutdown.load(Ordering::Relaxed) {
                // Process cleanup requests with priority
                select! {
                    request = rx.recv() => {
                        if let Some(req) = request {
                            pending.push(Reverse(req)); // Min heap by priority
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {
                        // Process pending cleanups
                        while let Some(Reverse(request)) = pending.pop() {
                            let start = Instant::now();
                            let result = request.cleanup.await;
                            
                            // Remove from active registry
                            active.write().await.remove(&request.resource_id);
                            
                            // Record cleanup metrics
                            monitor.record_event(MonitorEvent {
                                event_type: MonitorEventType::ResourceCleanup,
                                data: json!({
                                    "resource_id": request.resource_id,
                                    "success": result.is_ok(),
                                    "duration_ms": start.elapsed().as_millis(),
                                    "priority": request.priority,
                                }),
                            });
                            
                            break; // Process one per iteration
                        }
                    }
                }
            }
        });
    }
}
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum CleanupError {
    #[error("Cleanup operation timed out")]
    Timeout,
    
    #[error("Resource was already cleaned up")]
    AlreadyCleanedUp,
    
    #[error("Cleanup failed: {0}")]
    Failed(#[from] Box<dyn std::error::Error + Send + Sync>),
    
    #[error("Registry is shutting down")]
    ShuttingDown,
}

impl CleanupError {
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            CleanupError::Timeout => RecoveryStrategy::Retry,
            CleanupError::AlreadyCleanedUp => RecoveryStrategy::Fail,
            CleanupError::Failed(_) => RecoveryStrategy::Fallback,
            CleanupError::ShuttingDown => RecoveryStrategy::Fail,
        }
    }
}
```

## 3. Data Sharing Component

### Immutable Sharing with CoW

```rust
/// Copy-on-write shared data for read-heavy workloads
pub struct SharedData<T: Clone> {
    data: Arc<T>,
    metrics: Arc<SharingMetrics>,
}

impl<T: Clone> SharedData<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(data),
            metrics: Arc::new(SharingMetrics::new()),
        }
    }
    
    /// Get immutable reference to data (zero-cost)
    pub fn get(&self) -> &T {
        self.metrics.record_read();
        &self.data
    }
    
    /// Clone the shared reference (cheap Arc clone)
    pub fn clone_ref(&self) -> Self {
        self.metrics.record_clone();
        Self {
            data: Arc::clone(&self.data),
            metrics: Arc::clone(&self.metrics),
        }
    }
    
    /// Create a mutable version (copy-on-write)
    pub fn make_mut(&mut self) -> &mut T {
        self.metrics.record_write();
        Arc::make_mut(&mut self.data)
    }
    
    /// Update data with function (handles CoW automatically)
    pub fn update<F, R>(&mut self, f: F) -> R
    where F: FnOnce(&mut T) -> R {
        f(self.make_mut())
    }
    
    /// Try to update without cloning (only if exclusively owned)
    pub fn try_update<F, R>(&mut self, f: F) -> Result<R, TryUpdateError<F>>
    where F: FnOnce(&mut T) -> R {
        match Arc::get_mut(&mut self.data) {
            Some(data) => {
                self.metrics.record_exclusive_write();
                Ok(f(data))
            }
            None => Err(TryUpdateError::SharedOwnership(f)),
        }
    }
}

#[derive(Debug)]
pub enum TryUpdateError<F> {
    SharedOwnership(F),
}
```

### Mutable Shared Data

```rust
/// Reader-writer shared data for occasional writes
pub struct MutableSharedData<T> {
    data: Arc<RwLock<T>>,
    metrics: Arc<SharingMetrics>,
}

impl<T> MutableSharedData<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
            metrics: Arc::new(SharingMetrics::new()),
        }
    }
    
    /// Read with shared lock
    pub async fn read(&self) -> RwLockReadGuard<'_, T> {
        self.metrics.record_read();
        self.data.read().await
    }
    
    /// Write with exclusive lock
    pub async fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.metrics.record_write();
        self.data.write().await
    }
    
    /// Apply function with read lock
    pub async fn with_read<F, R>(&self, f: F) -> R
    where F: FnOnce(&T) -> R {
        f(&*self.read().await)
    }
    
    /// Apply function with write lock
    pub async fn with_write<F, R>(&self, f: F) -> R
    where F: FnOnce(&mut T) -> R {
        f(&mut *self.write().await)
    }
    
    /// Try to read without waiting (non-blocking)
    pub fn try_read(&self) -> Result<RwLockReadGuard<'_, T>, TryLockError> {
        self.data.try_read().map_err(|_| TryLockError::WouldBlock)
    }
    
    /// Try to write without waiting (non-blocking)
    pub fn try_write(&self) -> Result<RwLockWriteGuard<'_, T>, TryLockError> {
        self.data.try_write().map_err(|_| TryLockError::WouldBlock)
    }
}
```

### Configuration Sharing with Notifications

```rust
/// Configuration data with change notifications
pub struct ConfigData<T: Clone> {
    data: Arc<T>,
    subscribers: Arc<RwLock<Vec<watch::Sender<T>>>>,
    version: AtomicU64,
    metrics: Arc<SharingMetrics>,
}

impl<T: Clone> ConfigData<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(data),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            version: AtomicU64::new(1),
            metrics: Arc::new(SharingMetrics::new()),
        }
    }
    
    /// Get current configuration
    pub fn get(&self) -> &T {
        self.metrics.record_read();
        &self.data
    }
    
    /// Update configuration and notify subscribers
    pub async fn update(&mut self, new_data: T) -> Result<(), ConfigUpdateError> {
        // Update data
        self.data = Arc::new(new_data.clone());
        self.version.fetch_add(1, Ordering::Relaxed);
        self.metrics.record_write();
        
        // Notify all subscribers
        let subscribers = self.subscribers.read().await;
        let mut failed = Vec::new();
        
        for (i, sender) in subscribers.iter().enumerate() {
            if sender.send(new_data.clone()).is_err() {
                failed.push(i);
            }
        }
        
        // Clean up failed subscribers
        if !failed.is_empty() {
            drop(subscribers);
            let mut subscribers = self.subscribers.write().await;
            for &i in failed.iter().rev() {
                subscribers.remove(i);
            }
        }
        
        self.metrics.record_notification(subscribers.len() - failed.len());
        Ok(())
    }
    
    /// Subscribe to configuration changes
    pub async fn subscribe(&self) -> watch::Receiver<T> {
        let (tx, rx) = watch::channel((*self.data).clone());
        
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(tx);
        
        rx
    }
    
    /// Get current version number
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }
}
```

### Sharing Metrics

```rust
#[derive(Debug)]
pub struct SharingMetrics {
    reads: AtomicU64,
    writes: AtomicU64,
    clones: AtomicU64,
    exclusive_writes: AtomicU64,
    notifications: AtomicU64,
    created_at: Instant,
}

impl SharingMetrics {
    pub fn new() -> Self {
        Self {
            reads: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            clones: AtomicU64::new(0),
            exclusive_writes: AtomicU64::new(0),
            notifications: AtomicU64::new(0),
            created_at: Instant::now(),
        }
    }
    
    pub fn record_read(&self) {
        self.reads.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_write(&self) {
        self.writes.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_clone(&self) {
        self.clones.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_exclusive_write(&self) {
        self.exclusive_writes.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_notification(&self, count: usize) {
        self.notifications.fetch_add(count as u64, Ordering::Relaxed);
    }
    
    pub fn get_stats(&self) -> SharingStats {
        SharingStats {
            reads: self.reads.load(Ordering::Relaxed),
            writes: self.writes.load(Ordering::Relaxed),
            clones: self.clones.load(Ordering::Relaxed),
            exclusive_writes: self.exclusive_writes.load(Ordering::Relaxed),
            notifications: self.notifications.load(Ordering::Relaxed),
            age: self.created_at.elapsed(),
        }
    }
}
```

## 4. Memory Mapping Component

### Safe Memory Mapping Interface

```rust
/// Type-safe memory mapped file
pub struct MappedFile<T> {
    _file: File,
    mapping: Mmap,
    _phantom: PhantomData<T>,
}

/// Bounds-checked slice over mapped memory
pub struct MappedSlice<T> {
    data: NonNull<T>,
    len: usize,
    _guard: Arc<MappedFile<()>>, // Keep mapping alive
}

/// Trait for types that can be safely memory-mapped
pub trait MappableType: Copy + 'static {
    /// Validate that the data layout is correct for this type
    fn validate_layout(data: &[u8]) -> Result<usize, MappingError>;
    
    /// Check alignment requirements
    fn alignment_requirement() -> usize {
        std::mem::align_of::<Self>()
    }
}

// Implement for common types
impl MappableType for u8 {
    fn validate_layout(data: &[u8]) -> Result<usize, MappingError> {
        Ok(data.len())
    }
}

impl MappableType for u32 {
    fn validate_layout(data: &[u8]) -> Result<usize, MappingError> {
        if data.len() % 4 != 0 {
            return Err(MappingError::AlignmentMismatch);
        }
        if data.as_ptr() as usize % 4 != 0 {
            return Err(MappingError::AddressAlignment);
        }
        Ok(data.len() / 4)
    }
}
```

### Implementation

```rust
impl<T: MappableType> MappedFile<T> {
    /// Create a new memory mapping
    pub fn new(file: File) -> Result<Self, MappingError> {
        // Get file size
        let metadata = file.metadata().map_err(MappingError::IoError)?;
        let len = metadata.len() as usize;
        
        // Create memory mapping
        let mapping = unsafe {
            MmapOptions::new()
                .len(len)
                .map(&file)
                .map_err(MappingError::MmapError)?
        };
        
        // Validate type layout
        T::validate_layout(&mapping)?;
        
        Ok(Self {
            _file: file,
            mapping,
            _phantom: PhantomData,
        })
    }
    
    /// Get a typed slice over the mapped data
    pub fn as_slice(&self) -> Result<&[T], MappingError> {
        let len = T::validate_layout(&self.mapping)?;
        
        // Safety: We've validated the layout and alignment
        let ptr = self.mapping.as_ptr() as *const T;
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        
        Ok(slice)
    }
    
    /// Get a bounds-checked slice wrapper
    pub fn as_mapped_slice(&self) -> Result<MappedSlice<T>, MappingError> {
        let len = T::validate_layout(&self.mapping)?;
        let ptr = NonNull::new(self.mapping.as_ptr() as *mut T)
            .ok_or(MappingError::NullPointer)?;
        
        // Create type-erased guard to keep mapping alive
        let guard = Arc::new(unsafe { 
            std::mem::transmute::<MappedFile<T>, MappedFile<()>>(
                std::ptr::read(self as *const _)
            )
        });
        std::mem::forget(self); // Prevent double-drop
        
        Ok(MappedSlice {
            data: ptr,
            len,
            _guard: guard,
        })
    }
}

impl<T> MappedSlice<T> {
    /// Get element at index with bounds checking
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            Some(unsafe { self.data.as_ptr().add(index).as_ref().unwrap() })
        } else {
            None
        }
    }
    
    /// Get slice of elements with bounds checking
    pub fn slice(&self, range: Range<usize>) -> Option<&[T]> {
        if range.end <= self.len {
            let ptr = unsafe { self.data.as_ptr().add(range.start) };
            let slice = unsafe { 
                std::slice::from_raw_parts(ptr, range.end - range.start) 
            };
            Some(slice)
        } else {
            None
        }
    }
    
    /// Get the length of the mapped data
    pub fn len(&self) -> usize {
        self.len
    }
    
    /// Check if the mapping is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

// Safety: MappedSlice is safe to send between threads
unsafe impl<T> Send for MappedSlice<T> where T: Send {}
unsafe impl<T> Sync for MappedSlice<T> where T: Sync {}
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum MappingError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Memory mapping error: {0}")]
    MmapError(std::io::Error),
    
    #[error("Data alignment mismatch for type")]
    AlignmentMismatch,
    
    #[error("Address not properly aligned")]
    AddressAlignment,
    
    #[error("Null pointer in mapping")]
    NullPointer,
    
    #[error("Invalid file format for type")]
    InvalidFormat,
}
```

## 5. Caching Component

### Multi-Policy Cache Design

```rust
/// High-performance cache with pluggable policies
pub struct Cache<K, V> {
    inner: CacheInner<K, V>,
    config: CacheConfig,
    metrics: Arc<CacheMetrics>,
    _cleanup_task: JoinHandle<()>,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_entries: usize,
    pub max_size_bytes: Option<usize>,
    pub default_ttl: Option<Duration>,
    pub eviction_policy: EvictionPolicy,
    pub cleanup_interval: Duration,
}

/// Pluggable eviction policies
pub enum EvictionPolicy {
    /// Least Recently Used
    Lru,
    /// Least Frequently Used  
    Lfu,
    /// Time To Live only
    Ttl(Duration),
    /// Custom eviction strategy
    Custom(Arc<dyn EvictionStrategy<K, V>>),
}

/// Custom eviction strategy trait
pub trait EvictionStrategy<K, V>: Send + Sync {
    /// Should this entry be evicted?
    fn should_evict(&self, entry: &CacheEntry<K, V>) -> bool;
    
    /// Called when entry is accessed
    fn on_access(&self, entry: &mut CacheEntry<K, V>);
    
    /// Called when entry is inserted
    fn on_insert(&self, entry: &mut CacheEntry<K, V>);
    
    /// Priority for eviction (lower = evict first)
    fn eviction_priority(&self, entry: &CacheEntry<K, V>) -> u64;
}
```

### Cache Entry and Storage

```rust
/// Cache entry with metadata
pub struct CacheEntry<K, V> {
    pub key: K,
    pub value: V,
    pub inserted_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub expires_at: Option<Instant>,
    pub size_bytes: Option<usize>,
}

/// Internal cache storage (sharded for concurrency)
struct CacheInner<K, V> {
    shards: Vec<CacheShard<K, V>>,
    hasher: RandomState,
}

struct CacheShard<K, V> {
    data: RwLock<HashMap<K, CacheEntry<K, V>>>,
    lru_list: Mutex<LinkedList<K>>, // For LRU tracking
}

impl<K: Hash + Eq + Clone, V> Cache<K, V> {
    pub fn new(config: CacheConfig) -> Self {
        let shard_count = num_cpus::get().max(4);
        let shards = (0..shard_count)
            .map(|_| CacheShard::new())
            .collect();
        
        let inner = CacheInner {
            shards,
            hasher: RandomState::new(),
        };
        
        let metrics = Arc::new(CacheMetrics::new());
        let cleanup_task = Self::start_cleanup_task(&inner, &config, &metrics);
        
        Self {
            inner,
            config,
            metrics,
            _cleanup_task: cleanup_task,
        }
    }
    
    /// Get value from cache
    pub async fn get(&self, key: &K) -> Option<V> 
    where V: Clone {
        let shard_idx = self.shard_index(key);
        let shard = &self.inner.shards[shard_idx];
        
        let start = Instant::now();
        
        // Try read lock first
        if let Ok(data) = shard.data.try_read() {
            if let Some(entry) = data.get(key) {
                // Check TTL
                if let Some(expires) = entry.expires_at {
                    if Instant::now() > expires {
                        drop(data);
                        self.remove_expired(key).await;
                        self.metrics.record_miss(start.elapsed());
                        return None;
                    }
                }
                
                let value = entry.value.clone();
                drop(data);
                
                // Update access metadata asynchronously
                self.update_access_async(key, shard_idx).await;
                self.metrics.record_hit(start.elapsed());
                return Some(value);
            }
        }
        
        self.metrics.record_miss(start.elapsed());
        None
    }
    
    /// Insert value into cache
    pub async fn insert(&self, key: K, value: V) -> Option<V>
    where K: Clone, V: Clone {
        let shard_idx = self.shard_index(&key);
        let shard = &self.inner.shards[shard_idx];
        
        let now = Instant::now();
        let expires_at = self.config.default_ttl.map(|ttl| now + ttl);
        let size_bytes = self.estimate_size(&value);
        
        let mut entry = CacheEntry {
            key: key.clone(),
            value,
            inserted_at: now,
            last_accessed: now,
            access_count: 0,
            expires_at,
            size_bytes,
        };
        
        // Apply eviction policy
        match &self.config.eviction_policy {
            EvictionPolicy::Custom(strategy) => {
                strategy.on_insert(&mut entry);
            }
            _ => {} // Built-in policies handled elsewhere
        }
        
        let mut data = shard.data.write().await;
        
        // Check if eviction needed
        if data.len() >= self.config.max_entries {
            self.evict_entries(&mut data, 1).await;
        }
        
        let old_value = data.insert(key.clone(), entry)
            .map(|old| old.value);
        
        self.metrics.record_insert();
        old_value
    }
    
    /// Remove entry from cache
    pub async fn remove(&self, key: &K) -> Option<V> {
        let shard_idx = self.shard_index(key);
        let shard = &self.inner.shards[shard_idx];
        
        let mut data = shard.data.write().await;
        data.remove(key).map(|entry| {
            self.metrics.record_removal();
            entry.value
        })
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.metrics.get_stats()
    }
    
    /// Clear all entries
    pub async fn clear(&self) {
        for shard in &self.inner.shards {
            let mut data = shard.data.write().await;
            let removed = data.len();
            data.clear();
            self.metrics.record_bulk_removal(removed);
        }
    }
    
    fn shard_index(&self, key: &K) -> usize {
        let hash = self.inner.hasher.hash_one(key);
        hash as usize % self.inner.shards.len()
    }
    
    fn estimate_size<T>(&self, _value: &T) -> Option<usize> {
        // Simplified size estimation
        Some(std::mem::size_of::<T>())
    }
    
    async fn evict_entries<'a>(&self, data: &mut HashMap<K, CacheEntry<K, V>>, count: usize) 
    where K: Clone {
        match &self.config.eviction_policy {
            EvictionPolicy::Lru => {
                // Find least recently used entries
                let mut entries: Vec<_> = data.iter().collect();
                entries.sort_by_key(|(_, entry)| entry.last_accessed);
                
                for (key, _) in entries.into_iter().take(count) {
                    data.remove(key);
                    self.metrics.record_eviction();
                }
            }
            EvictionPolicy::Lfu => {
                // Find least frequently used entries  
                let mut entries: Vec<_> = data.iter().collect();
                entries.sort_by_key(|(_, entry)| entry.access_count);
                
                for (key, _) in entries.into_iter().take(count) {
                    data.remove(key);
                    self.metrics.record_eviction();
                }
            }
            EvictionPolicy::Custom(strategy) => {
                // Use custom strategy priorities
                let mut entries: Vec<_> = data.iter().collect();
                entries.sort_by_key(|(_, entry)| strategy.eviction_priority(entry));
                
                for (key, _) in entries.into_iter().take(count) {
                    data.remove(key);
                    self.metrics.record_eviction();
                }
            }
            EvictionPolicy::Ttl(_) => {
                // TTL handled by cleanup task, random eviction here
                let keys: Vec<_> = data.keys().take(count).cloned().collect();
                for key in keys {
                    data.remove(&key);
                    self.metrics.record_eviction();
                }
            }
        }
    }
    
    async fn update_access_async(&self, key: &K, shard_idx: usize) 
    where K: Clone {
        let shard = &self.inner.shards[shard_idx];
        
        if let Ok(mut data) = shard.data.try_write() {
            if let Some(entry) = data.get_mut(key) {
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                
                // Apply custom policy
                if let EvictionPolicy::Custom(strategy) = &self.config.eviction_policy {
                    strategy.on_access(entry);
                }
            }
        }
        // If we can't get write lock, skip update (not critical)
    }
    
    fn start_cleanup_task(
        inner: &CacheInner<K, V>,
        config: &CacheConfig,
        metrics: &Arc<CacheMetrics>,
    ) -> JoinHandle<()> 
    where K: Clone + Send + Sync + 'static, V: Send + Sync + 'static {
        let shards = inner.shards.clone();
        let cleanup_interval = config.cleanup_interval;
        let metrics = Arc::clone(metrics);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                
                let now = Instant::now();
                let mut total_expired = 0;
                
                for shard in &shards {
                    if let Ok(mut data) = shard.data.try_write() {
                        let expired_keys: Vec<_> = data
                            .iter()
                            .filter_map(|(k, v)| {
                                if let Some(expires) = v.expires_at {
                                    if now > expires {
                                        Some(k.clone())
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect();
                        
                        for key in expired_keys {
                            data.remove(&key);
                            total_expired += 1;
                        }
                    }
                }
                
                if total_expired > 0 {
                    metrics.record_cleanup(total_expired);
                }
            }
        })
    }
}
```

### Cache Metrics

```rust
#[derive(Debug)]
pub struct CacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    inserts: AtomicU64,
    removals: AtomicU64,
    evictions: AtomicU64,
    expired_cleanups: AtomicU64,
    total_hit_time: AtomicU64,  // nanoseconds
    total_miss_time: AtomicU64, // nanoseconds
    created_at: Instant,
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub inserts: u64,
    pub removals: u64,
    pub evictions: u64,
    pub expired_cleanups: u64,
    pub avg_hit_time_ns: f64,
    pub avg_miss_time_ns: f64,
    pub age: Duration,
}

impl CacheMetrics {
    pub fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            inserts: AtomicU64::new(0),
            removals: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            expired_cleanups: AtomicU64::new(0),
            total_hit_time: AtomicU64::new(0),
            total_miss_time: AtomicU64::new(0),
            created_at: Instant::now(),
        }
    }
    
    pub fn record_hit(&self, duration: Duration) {
        self.hits.fetch_add(1, Ordering::Relaxed);
        self.total_hit_time.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }
    
    pub fn record_miss(&self, duration: Duration) {
        self.misses.fetch_add(1, Ordering::Relaxed);
        self.total_miss_time.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }
    
    pub fn get_stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total_requests = hits + misses;
        
        let hit_rate = if total_requests > 0 {
            hits as f64 / total_requests as f64
        } else {
            0.0
        };
        
        let avg_hit_time_ns = if hits > 0 {
            self.total_hit_time.load(Ordering::Relaxed) as f64 / hits as f64
        } else {
            0.0
        };
        
        let avg_miss_time_ns = if misses > 0 {
            self.total_miss_time.load(Ordering::Relaxed) as f64 / misses as f64
        } else {
            0.0
        };
        
        CacheStats {
            hits,
            misses,
            hit_rate,
            inserts: self.inserts.load(Ordering::Relaxed),
            removals: self.removals.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            expired_cleanups: self.expired_cleanups.load(Ordering::Relaxed),
            avg_hit_time_ns,
            avg_miss_time_ns,
            age: self.created_at.elapsed(),
        }
    }
}