//! RPC response caching module
//!
//! Provides thread-safe caching for frequently requested Solana RPC data
//! with configurable TTL (time-to-live) and size limits.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::Arc;
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;

/// Configuration for the RPC cache
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CacheConfig {
    /// Whether caching is enabled
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    /// Maximum number of entries in the cache
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
    /// Default TTL for cache entries in seconds
    #[serde(default = "default_ttl_seconds")]
    pub default_ttl_seconds: u64,
    /// TTL overrides for specific methods (method_name -> ttl_seconds)
    #[serde(default)]
    pub method_ttl_overrides: std::collections::HashMap<String, u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            max_entries: default_max_entries(),
            default_ttl_seconds: default_ttl_seconds(),
            method_ttl_overrides: default_method_ttl_overrides(),
        }
    }
}

fn default_cache_enabled() -> bool {
    true
}

fn default_max_entries() -> usize {
    10000
}

fn default_ttl_seconds() -> u64 {
    30 // 30 seconds default
}

fn default_method_ttl_overrides() -> std::collections::HashMap<String, u64> {
    let mut overrides = std::collections::HashMap::new();
    // Block data is relatively stable once finalized
    overrides.insert("getBlock".to_string(), 300); // 5 minutes
    overrides.insert("getBlockTime".to_string(), 300);
    overrides.insert("getBlockHeight".to_string(), 5); // Changes frequently
    
    // Account data changes less frequently for most accounts
    overrides.insert("getAccountInfo".to_string(), 10);
    overrides.insert("getBalance".to_string(), 10);
    overrides.insert("getMultipleAccounts".to_string(), 10);
    
    // Token data is relatively stable
    overrides.insert("getTokenAccountBalance".to_string(), 30);
    overrides.insert("getTokenSupply".to_string(), 60);
    
    // System info changes infrequently
    overrides.insert("getGenesisHash".to_string(), 3600); // 1 hour
    overrides.insert("getVersion".to_string(), 600); // 10 minutes
    overrides.insert("getEpochSchedule".to_string(), 3600);
    
    // Current slot changes rapidly
    overrides.insert("getSlot".to_string(), 2);
    
    overrides
}

/// A cached entry with expiration time
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached value
    value: serde_json::Value,
    /// When this entry was created
    created_at: Instant,
    /// How long this entry is valid for
    ttl: Duration,
}

impl CacheEntry {
    /// Check if this cache entry has expired
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Thread-safe RPC response cache
pub struct RpcCache {
    /// The cache storage
    cache: Arc<DashMap<u64, CacheEntry>>,
    /// Cache configuration
    config: CacheConfig,
}

impl RpcCache {
    /// Create a new RPC cache with the given configuration
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Generate a cache key from method name and parameters
    /// 
    /// Note: Uses JSON string representation for hashing. While not guaranteed to be
    /// stable across different JSON implementations, in practice serde_json provides
    /// consistent serialization for identical data structures. Clients should ensure
    /// consistent parameter ordering for optimal cache hit rates.
    fn generate_key(method: &str, params: &serde_json::Value) -> u64 {
        let mut hasher = DefaultHasher::new();
        method.hash(&mut hasher);
        // Use JSON string representation for hashing
        params.to_string().hash(&mut hasher);
        hasher.finish()
    }

    /// Get the TTL for a specific method
    fn get_ttl_for_method(&self, method: &str) -> Duration {
        let seconds = self.config.method_ttl_overrides
            .get(method)
            .copied()
            .unwrap_or(self.config.default_ttl_seconds);
        Duration::from_secs(seconds)
    }

    /// Get a value from the cache
    ///
    /// Returns None if the entry doesn't exist or has expired
    pub fn get(&self, method: &str, params: &serde_json::Value) -> Option<serde_json::Value> {
        if !self.config.enabled {
            return None;
        }

        let key = Self::generate_key(method, params);
        
        if let Some(entry) = self.cache.get(&key) {
            if entry.is_expired() {
                // Remove expired entry
                drop(entry);
                self.cache.remove(&key);
                return None;
            }
            return Some(entry.value.clone());
        }
        
        None
    }

    /// Store a value in the cache
    pub fn set(&self, method: &str, params: &serde_json::Value, value: serde_json::Value) {
        if !self.config.enabled {
            return;
        }

        let key = Self::generate_key(method, params);
        
        // Check if this key already exists - if so, we're just updating
        let is_update = self.cache.contains_key(&key);
        
        // Only enforce size limit for new entries
        // Note: There's a potential race condition where multiple threads could pass
        // this check simultaneously and each evict an entry. This is acceptable as
        // the cache may temporarily exceed max_entries by a small amount in high
        // concurrency scenarios, but will self-correct on subsequent operations.
        if !is_update && self.cache.len() >= self.config.max_entries {
            // Simple eviction: collect first key and remove it (FIFO)
            // PERFORMANCE NOTE: This uses a basic FIFO eviction strategy (removes first entry)
            // which is simple and fast but not optimal for cache efficiency.
            // For production workloads with high cache pressure, consider implementing:
            // - LRU (Least Recently Used): Better cache hit rates
            // - LFU (Least Frequently Used): Good for skewed access patterns
            // - TTL-aware eviction: Evict entries closest to expiration first
            // Current strategy is adequate for most use cases and avoids performance overhead
            let evict_key = self.cache.iter().next().map(|entry| *entry.key());
            if let Some(k) = evict_key {
                self.cache.remove(&k);
            }
        }

        let ttl = self.get_ttl_for_method(method);
        
        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
            ttl,
        };

        self.cache.insert(key, entry);
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get the current size of the cache
    pub fn size(&self) -> usize {
        self.cache.len()
    }

    /// Remove expired entries from the cache
    pub fn evict_expired(&self) {
        self.cache.retain(|_, entry| !entry.is_expired());
    }

    /// Get cache hit rate statistics (for metrics)
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Global RPC cache instance
pub static GLOBAL_RPC_CACHE: Lazy<RpcCache> = Lazy::new(|| {
    RpcCache::new(CacheConfig::default())
});

/// Helper function to execute an RPC call with caching
///
/// This function checks the cache first, and if not found, executes the provided
/// async function and caches the result.
///
/// # Arguments
/// * `cache` - The cache instance to use
/// * `method` - The RPC method name
/// * `params` - The parameters for the RPC call (used for cache key generation)
/// * `f` - The async function to execute if cache misses
///
/// # Returns
/// * `Ok(Value)` - The cached or freshly fetched result
/// * `Err(E)` - Any error from the RPC call
pub async fn with_cache<F, Fut, E>(
    cache: &RpcCache,
    method: &str,
    params: &serde_json::Value,
    f: F,
) -> Result<serde_json::Value, E>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<serde_json::Value, E>>,
{
    // Try to get from cache first
    if let Some(cached_value) = cache.get(method, params) {
        // Record cache hit in metrics
        crate::metrics::PROMETHEUS_METRICS.record_cache_hit(method);
        return Ok(cached_value);
    }

    // Record cache miss in metrics
    crate::metrics::PROMETHEUS_METRICS.record_cache_miss(method);

    // Cache miss - execute the function
    let result = f().await?;

    // Store in cache
    cache.set(method, params, result.clone());

    // Update cache size metric
    crate::metrics::PROMETHEUS_METRICS.update_cache_size("rpc", cache.size());

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let config = CacheConfig {
            enabled: true,
            max_entries: 100,
            default_ttl_seconds: 10,
            method_ttl_overrides: std::collections::HashMap::new(),
        };
        let cache = RpcCache::new(config);

        let method = "getBalance";
        let params = serde_json::json!({"pubkey": "test123"});
        let value = serde_json::json!({"balance": 1000});

        // Initially, cache should be empty
        assert!(cache.get(method, &params).is_none());

        // Set a value
        cache.set(method, &params, value.clone());

        // Should be able to retrieve it
        assert_eq!(cache.get(method, &params), Some(value));

        // Different params should not match
        let different_params = serde_json::json!({"pubkey": "test456"});
        assert!(cache.get(method, &different_params).is_none());
    }

    #[test]
    fn test_cache_expiration() {
        let config = CacheConfig {
            enabled: true,
            max_entries: 100,
            default_ttl_seconds: 1, // 1 second TTL for testing
            method_ttl_overrides: std::collections::HashMap::new(),
        };
        let cache = RpcCache::new(config);

        let method = "getBalance";
        let params = serde_json::json!({"pubkey": "test123"});
        let value = serde_json::json!({"balance": 1000});

        cache.set(method, &params, value.clone());
        
        // Should be available immediately
        assert!(cache.get(method, &params).is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(1100));

        // Should be expired now
        assert!(cache.get(method, &params).is_none());
    }

    #[test]
    fn test_cache_disabled() {
        let config = CacheConfig {
            enabled: false,
            max_entries: 100,
            default_ttl_seconds: 10,
            method_ttl_overrides: std::collections::HashMap::new(),
        };
        let cache = RpcCache::new(config);

        let method = "getBalance";
        let params = serde_json::json!({"pubkey": "test123"});
        let value = serde_json::json!({"balance": 1000});

        cache.set(method, &params, value.clone());

        // Cache is disabled, so get should always return None
        assert!(cache.get(method, &params).is_none());
    }

    #[test]
    fn test_method_specific_ttl() {
        let mut overrides = std::collections::HashMap::new();
        overrides.insert("getGenesisHash".to_string(), 3600);
        
        let config = CacheConfig {
            enabled: true,
            max_entries: 100,
            default_ttl_seconds: 10,
            method_ttl_overrides: overrides,
        };
        let cache = RpcCache::new(config);

        // Genesis hash should have 3600 second TTL
        let ttl = cache.get_ttl_for_method("getGenesisHash");
        assert_eq!(ttl, Duration::from_secs(3600));

        // Other methods should use default
        let ttl = cache.get_ttl_for_method("getBalance");
        assert_eq!(ttl, Duration::from_secs(10));
    }

    #[test]
    fn test_cache_size_limit() {
        let config = CacheConfig {
            enabled: true,
            max_entries: 3,
            default_ttl_seconds: 60,
            method_ttl_overrides: std::collections::HashMap::new(),
        };
        let cache = RpcCache::new(config);

        // Fill cache to limit with unique keys
        for i in 0..3 {
            let params = serde_json::json!({"pubkey": format!("addr_{}", i)});
            cache.set("getBalance", &params, serde_json::json!({"balance": i * 100}));
        }

        assert_eq!(cache.size(), 3);

        // Adding one more with a completely different key should trigger eviction
        let new_params = serde_json::json!({"pubkey": "new_address_xyz"});
        cache.set("getBalance", &new_params, serde_json::json!({"balance": 999}));
        
        // Size should still be at the limit
        assert_eq!(cache.size(), 3);
    }

    #[test]
    fn test_evict_expired() {
        let config = CacheConfig {
            enabled: true,
            max_entries: 100,
            default_ttl_seconds: 1,
            method_ttl_overrides: std::collections::HashMap::new(),
        };
        let cache = RpcCache::new(config);

        // Add some entries
        for i in 0..5 {
            let params = serde_json::json!({"pubkey": format!("test{}", i)});
            cache.set("getBalance", &params, serde_json::json!({"balance": i}));
        }

        assert_eq!(cache.size(), 5);

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(1100));

        // Manually evict expired entries
        cache.evict_expired();

        assert_eq!(cache.size(), 0);
    }

    #[tokio::test]
    async fn test_with_cache_helper() {
        let config = CacheConfig {
            enabled: true,
            max_entries: 100,
            default_ttl_seconds: 10,
            method_ttl_overrides: std::collections::HashMap::new(),
        };
        let cache = RpcCache::new(config);

        let method = "getBalance";
        let params = serde_json::json!({"pubkey": "test123"});

        // First call should miss cache and execute function
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();
        
        let result1 = with_cache(&cache, method, &params, || {
            let count = call_count_clone.clone();
            async move {
                count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok::<_, String>(serde_json::json!({"balance": 1000}))
            }
        }).await.unwrap();

        assert_eq!(result1, serde_json::json!({"balance": 1000}));
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);

        // Second call should hit cache and not execute function
        let result2 = with_cache(&cache, method, &params, || {
            let count = call_count.clone();
            async move {
                count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok::<_, String>(serde_json::json!({"balance": 1000}))
            }
        }).await.unwrap();

        assert_eq!(result2, serde_json::json!({"balance": 1000}));
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1); // Should still be 1
    }
}
