# RPC Response Caching

The Solana MCP Server includes an automated caching mechanism for frequent RPC calls to reduce latency and improve performance during peak loads.

## Overview

The caching system provides:
- **TTL-based expiration**: Configurable time-to-live for cached responses
- **Method-specific TTLs**: Different cache durations for different RPC methods
- **Thread-safe access**: Concurrent cache access using DashMap
- **Size limits**: Automatic eviction when cache reaches capacity
- **Prometheus metrics**: Cache hit/miss tracking and size monitoring

## Configuration

Cache configuration is managed through the `config.json` file or can be set programmatically:

```json
{
  "rpc_url": "https://api.opensvm.com",
  "commitment": "confirmed",
  "protocol_version": "2025-06-18",
  "cache": {
    "enabled": true,
    "max_entries": 10000,
    "default_ttl_seconds": 30,
    "method_ttl_overrides": {
      "getBlock": 300,
      "getBlockTime": 300,
      "getBlockHeight": 5,
      "getAccountInfo": 10,
      "getBalance": 10,
      "getMultipleAccounts": 10,
      "getTokenAccountBalance": 30,
      "getTokenSupply": 60,
      "getGenesisHash": 3600,
      "getVersion": 600,
      "getEpochSchedule": 3600,
      "getSlot": 2
    }
  }
}
```

### Configuration Options

#### `enabled` (boolean, default: `true`)
Whether caching is enabled. Set to `false` to disable caching entirely.

#### `max_entries` (integer, default: `10000`)
Maximum number of entries the cache can hold. When this limit is reached, the oldest entry will be evicted to make room for new ones.

#### `default_ttl_seconds` (integer, default: `30`)
Default time-to-live in seconds for cached responses. This is used for methods that don't have a specific TTL override.

#### `method_ttl_overrides` (object, default: see above)
A map of RPC method names to their specific TTL values in seconds. This allows fine-tuning cache duration based on how frequently the data changes.

## Default TTL Values

The system includes intelligent default TTL values based on the nature of each RPC method:

| Method | Default TTL | Reason |
|--------|-------------|---------|
| `getBlock` | 300s (5 min) | Block data is immutable once finalized |
| `getBlockTime` | 300s (5 min) | Block time doesn't change |
| `getBlockHeight` | 5s | Changes frequently as new blocks are produced |
| `getAccountInfo` | 10s | Account data can change, but not too frequently for most accounts |
| `getBalance` | 10s | Balances change periodically |
| `getMultipleAccounts` | 10s | Similar to single account info |
| `getTokenAccountBalance` | 30s | Token balances are relatively stable |
| `getTokenSupply` | 60s (1 min) | Total supply changes slowly |
| `getGenesisHash` | 3600s (1 hour) | Genesis hash never changes |
| `getVersion` | 600s (10 min) | Node version changes rarely |
| `getEpochSchedule` | 3600s (1 hour) | Epoch schedule is fixed |
| `getSlot` | 2s | Current slot changes rapidly |

## Cached Methods

The following RPC methods have cached versions available:

### Account Methods
- `getBalance` / `getBalanceCached`
- `getAccountInfo` / `getAccountInfoCached`

### System Methods
- `getVersion` / `getVersionCached`

More methods can be easily added by following the same pattern.

## Usage Examples

### Using the Cache Programmatically

```rust
use solana_mcp_server::{RpcCache, CacheConfig, with_cache};
use std::sync::Arc;

// Create a cache with custom configuration
let config = CacheConfig {
    enabled: true,
    max_entries: 5000,
    default_ttl_seconds: 30,
    method_ttl_overrides: std::collections::HashMap::new(),
};

let cache = Arc::new(RpcCache::new(config));

// Use the with_cache helper for automatic caching
let result = with_cache(&cache, "getBalance", &params, || async {
    // Your RPC call logic here
    client.get_balance(&pubkey).await
}).await?;
```

### Using Cached RPC Methods

```rust
use solana_mcp_server::rpc::accounts;

// Use the cached version
let balance = accounts::get_balance_cached(&client, &pubkey, &cache).await?;

// Or use the non-cached version
let balance = accounts::get_balance(&client, &pubkey).await?;
```

## Metrics

The cache exposes the following Prometheus metrics:

### `solana_mcp_cache_hits_total`
Counter tracking cache hits by method name.

**Labels:**
- `method`: The RPC method name

### `solana_mcp_cache_misses_total`
Counter tracking cache misses by method name.

**Labels:**
- `method`: The RPC method name

### `solana_mcp_cache_size`
Gauge showing the current number of entries in the cache.

**Labels:**
- `cache_type`: Type of cache (currently "rpc")

## Performance Considerations

### When to Enable Caching

Caching is most beneficial when:
- You have repeated requests for the same data
- Your application queries relatively stable data (blocks, genesis hash, etc.)
- You want to reduce load on the RPC endpoint
- You can tolerate slightly stale data within the TTL window

### When to Disable Caching

Consider disabling caching when:
- You need real-time data for every request
- Your queries are highly unique (low cache hit rate)
- Memory is constrained
- Data changes very frequently

### Tuning for Your Use Case

1. **Monitor cache metrics**: Watch the hit/miss ratio to understand cache effectiveness
2. **Adjust TTLs**: Increase TTLs for more stable data, decrease for rapidly changing data
3. **Set appropriate size limits**: Balance memory usage with cache coverage
4. **Method-specific tuning**: Use TTL overrides for methods that have specific requirements

## Security Considerations

- The cache respects the same security validations as non-cached calls
- All RPC URLs must be HTTPS
- Sensitive data in cache keys is hashed, not stored directly
- Cache entries are automatically evicted after TTL expiration

## Implementation Details

### Cache Key Generation

Cache keys are generated by hashing the combination of:
1. RPC method name
2. Request parameters (as JSON string)

This ensures that identical requests hit the same cache entry.

### Eviction Policy

When the cache reaches its size limit:
1. The first entry (FIFO) is selected for eviction
2. The new entry is inserted
3. Expired entries are automatically removed on access

**Note:** In high-concurrency scenarios, the cache may temporarily exceed `max_entries` by a small amount due to race conditions between size checks. This is acceptable and will self-correct on subsequent operations.

### Thread Safety

The cache uses `DashMap` for concurrent access, making it safe to use from multiple threads without external locking.

## Testing

The cache implementation includes comprehensive tests:

- Unit tests in `src/cache.rs`
- Integration tests in `tests/cache_integration.rs`
- All tests validate:
  - Basic get/set operations
  - TTL expiration
  - Size limits and eviction
  - Method-specific TTLs
  - Cache enable/disable
  - Thread safety
  - The `with_cache` helper function

Run tests with:
```bash
cargo test cache
```

## Future Enhancements

Potential improvements for the caching system:

1. **LRU eviction**: Replace random eviction with Least Recently Used policy
2. **Persistent cache**: Option to persist cache to disk
3. **Cache warming**: Pre-populate cache with frequently requested data
4. **Distributed caching**: Support for Redis or similar distributed cache backends
5. **Advanced metrics**: Cache efficiency scores, per-method hit rates
6. **Dynamic TTL adjustment**: Automatically adjust TTLs based on data volatility
7. **Selective caching**: Allow users to specify which methods to cache via configuration
