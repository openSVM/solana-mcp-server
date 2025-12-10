/// Integration tests for RPC caching functionality
use solana_mcp_server::{CacheConfig, RpcCache};
use std::sync::Arc;

#[tokio::test]
async fn test_cache_integration_basic() {
    let config = CacheConfig {
        enabled: true,
        max_entries: 100,
        default_ttl_seconds: 60,
        method_ttl_overrides: std::collections::HashMap::new(),
    };
    
    let cache = Arc::new(RpcCache::new(config));
    
    // Verify cache is initially empty
    assert_eq!(cache.size(), 0);
    
    // Test cache miss
    let params = serde_json::json!({"test": "value"});
    assert!(cache.get("testMethod", &params).is_none());
    
    // Set a value
    let value = serde_json::json!({"result": "success"});
    cache.set("testMethod", &params, value.clone());
    
    // Verify cache hit
    assert_eq!(cache.get("testMethod", &params), Some(value));
    assert_eq!(cache.size(), 1);
}

#[tokio::test]
async fn test_cache_ttl_behavior() {
    let config = CacheConfig {
        enabled: true,
        max_entries: 100,
        default_ttl_seconds: 1, // 1 second TTL
        method_ttl_overrides: std::collections::HashMap::new(),
    };
    
    let cache = Arc::new(RpcCache::new(config));
    
    let params = serde_json::json!({"key": "value"});
    let value = serde_json::json!({"data": "test"});
    
    cache.set("shortLivedMethod", &params, value.clone());
    
    // Should be available immediately
    assert!(cache.get("shortLivedMethod", &params).is_some());
    
    // Wait for expiration
    tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;
    
    // Should be expired
    assert!(cache.get("shortLivedMethod", &params).is_none());
}

#[tokio::test]
async fn test_cache_method_specific_ttl() {
    let mut overrides = std::collections::HashMap::new();
    overrides.insert("longLivedMethod".to_string(), 300);
    overrides.insert("shortLivedMethod".to_string(), 1);
    
    let config = CacheConfig {
        enabled: true,
        max_entries: 100,
        default_ttl_seconds: 30,
        method_ttl_overrides: overrides,
    };
    
    let cache = Arc::new(RpcCache::new(config));
    
    // Set values for different methods
    let params1 = serde_json::json!({"id": 1});
    let params2 = serde_json::json!({"id": 2});
    
    cache.set("longLivedMethod", &params1, serde_json::json!({"result": "long"}));
    cache.set("shortLivedMethod", &params2, serde_json::json!({"result": "short"}));
    
    assert_eq!(cache.size(), 2);
    
    // Wait for short-lived to expire
    tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;
    
    // Long-lived should still be available
    assert!(cache.get("longLivedMethod", &params1).is_some());
    // Short-lived should be expired
    assert!(cache.get("shortLivedMethod", &params2).is_none());
}

#[tokio::test]
async fn test_cache_disabled() {
    let config = CacheConfig {
        enabled: false,
        max_entries: 100,
        default_ttl_seconds: 60,
        method_ttl_overrides: std::collections::HashMap::new(),
    };
    
    let cache = Arc::new(RpcCache::new(config));
    
    let params = serde_json::json!({"test": "disabled"});
    let value = serde_json::json!({"result": "data"});
    
    cache.set("testMethod", &params, value);
    
    // Even though we set a value, cache should not return it when disabled
    assert!(cache.get("testMethod", &params).is_none());
    assert!(!cache.is_enabled());
}

#[tokio::test]
async fn test_cache_with_different_params() {
    let config = CacheConfig {
        enabled: true,
        max_entries: 100,
        default_ttl_seconds: 60,
        method_ttl_overrides: std::collections::HashMap::new(),
    };
    
    let cache = Arc::new(RpcCache::new(config));
    
    // Same method, different params should be cached separately
    let params1 = serde_json::json!({"pubkey": "address1"});
    let params2 = serde_json::json!({"pubkey": "address2"});
    
    let value1 = serde_json::json!({"balance": 1000});
    let value2 = serde_json::json!({"balance": 2000});
    
    cache.set("getBalance", &params1, value1.clone());
    cache.set("getBalance", &params2, value2.clone());
    
    assert_eq!(cache.size(), 2);
    assert_eq!(cache.get("getBalance", &params1), Some(value1));
    assert_eq!(cache.get("getBalance", &params2), Some(value2));
}

#[tokio::test]
async fn test_cache_eviction() {
    let config = CacheConfig {
        enabled: true,
        max_entries: 3,
        default_ttl_seconds: 60,
        method_ttl_overrides: std::collections::HashMap::new(),
    };
    
    let cache = Arc::new(RpcCache::new(config));
    
    // Fill cache to capacity
    for i in 0..3 {
        let params = serde_json::json!({"id": i});
        let value = serde_json::json!({"data": i});
        cache.set("testMethod", &params, value);
    }
    
    assert_eq!(cache.size(), 3);
    
    // Adding one more should trigger eviction
    let params_overflow = serde_json::json!({"id": 999});
    cache.set("testMethod", &params_overflow, serde_json::json!({"data": 999}));
    
    // Size should still be at limit
    assert_eq!(cache.size(), 3);
}

#[tokio::test]
async fn test_cache_update_existing_entry() {
    let config = CacheConfig {
        enabled: true,
        max_entries: 100,
        default_ttl_seconds: 60,
        method_ttl_overrides: std::collections::HashMap::new(),
    };
    
    let cache = Arc::new(RpcCache::new(config));
    
    let params = serde_json::json!({"key": "update_test"});
    
    // Set initial value
    cache.set("testMethod", &params, serde_json::json!({"version": 1}));
    assert_eq!(cache.size(), 1);
    
    // Update the same entry
    cache.set("testMethod", &params, serde_json::json!({"version": 2}));
    
    // Size should still be 1 (update, not insert)
    assert_eq!(cache.size(), 1);
    
    // Should get the updated value
    let cached = cache.get("testMethod", &params).unwrap();
    assert_eq!(cached["version"], 2);
}

#[tokio::test]
async fn test_cache_clear() {
    let config = CacheConfig {
        enabled: true,
        max_entries: 100,
        default_ttl_seconds: 60,
        method_ttl_overrides: std::collections::HashMap::new(),
    };
    
    let cache = Arc::new(RpcCache::new(config));
    
    // Add multiple entries
    for i in 0..5 {
        let params = serde_json::json!({"id": i});
        cache.set("testMethod", &params, serde_json::json!({"data": i}));
    }
    
    assert_eq!(cache.size(), 5);
    
    // Clear cache
    cache.clear();
    
    assert_eq!(cache.size(), 0);
}

#[tokio::test]
async fn test_with_cache_helper() {
    use solana_mcp_server::with_cache;
    
    let config = CacheConfig {
        enabled: true,
        max_entries: 100,
        default_ttl_seconds: 60,
        method_ttl_overrides: std::collections::HashMap::new(),
    };
    
    let cache = RpcCache::new(config);
    let call_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
    
    let method = "testMethod";
    let params = serde_json::json!({"test": "param"});
    
    // First call should miss cache
    let call_count_clone = call_count.clone();
    let result1 = with_cache(&cache, method, &params, || {
        let count = call_count_clone.clone();
        async move {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok::<_, String>(serde_json::json!({"result": "success"}))
        }
    })
    .await
    .unwrap();
    
    assert_eq!(result1, serde_json::json!({"result": "success"}));
    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    
    // Second call should hit cache
    let result2 = with_cache(&cache, method, &params, || {
        let count = call_count.clone();
        async move {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok::<_, String>(serde_json::json!({"result": "success"}))
        }
    })
    .await
    .unwrap();
    
    assert_eq!(result2, serde_json::json!({"result": "success"}));
    // Counter should still be 1 (function not called again)
    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
}
