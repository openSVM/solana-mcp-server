-- Solana Validator Performance History
CREATE TABLE IF NOT EXISTS default.validator_performance (
    timestamp DateTime DEFAULT now(),
    vote_pubkey String,
    node_pubkey String,
    activated_stake UInt64,
    commission UInt8,
    epoch_credits UInt64,
    last_vote UInt64,
    root_slot UInt64,
    skip_rate Float32,
    is_delinquent UInt8,
    version String DEFAULT ''
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (timestamp, vote_pubkey);

-- TPS and Network Metrics
CREATE TABLE IF NOT EXISTS default.network_metrics (
    timestamp DateTime DEFAULT now(),
    slot UInt64,
    block_height UInt64,
    tps UInt32,
    transaction_count UInt64,
    total_validators UInt32,
    active_validators UInt32,
    delinquent_validators UInt32,
    total_stake UInt64
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY timestamp;

-- Transaction Activity by Program
CREATE TABLE IF NOT EXISTS default.program_activity (
    timestamp DateTime DEFAULT now(),
    program_id String,
    transaction_count UInt64,
    success_rate Float32,
    avg_compute_units UInt64
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (timestamp, program_id);

-- Token Analytics (Pump.fun, etc)
CREATE TABLE IF NOT EXISTS default.token_analytics (
    timestamp DateTime DEFAULT now(),
    token_mint String,
    holder_count UInt32,
    market_cap Float64,
    liquidity Float64,
    volume_24h Float64,
    price_change_24h Float32
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (timestamp, token_mint);

-- Materialized views for fast queries
CREATE MATERIALIZED VIEW IF NOT EXISTS default.validator_performance_hourly
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(hour)
ORDER BY (hour, vote_pubkey)
AS SELECT
    toStartOfHour(timestamp) as hour,
    vote_pubkey,
    avg(skip_rate) as avg_skip_rate,
    avg(activated_stake) as avg_stake,
    avg(commission) as avg_commission,
    sum(is_delinquent) as delinquent_count
FROM default.validator_performance
GROUP BY hour, vote_pubkey;

CREATE MATERIALIZED VIEW IF NOT EXISTS default.network_metrics_hourly
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(hour)
ORDER BY hour
AS SELECT
    toStartOfHour(timestamp) as hour,
    avg(tps) as avg_tps,
    max(tps) as max_tps,
    min(tps) as min_tps,
    avg(active_validators) as avg_active_validators
FROM default.network_metrics
GROUP BY hour;
