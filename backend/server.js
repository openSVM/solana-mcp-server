const express = require('express');
const cors = require('cors');
const axios = require('axios');

const app = express();
const PORT = 3001;

const CLICKHOUSE_URL = 'http://localhost:8123';
const MCP_URL = 'http://localhost:3000/api/mcp';

app.use(cors());
app.use(express.json());

// Helper: Query ClickHouse
async function queryClickHouse(query) {
    try {
        const response = await axios.post(CLICKHOUSE_URL, query, {
            params: { default_format: 'JSONCompact' }
        });
        return response.data;
    } catch (error) {
        console.error('ClickHouse query error:', error.message);
        throw error;
    }
}

// Helper: Call MCP
async function callMCP(method, params = {}) {
    try {
        const response = await axios.post(MCP_URL, {
            jsonrpc: '2.0',
            method: 'tools/call',
            id: Date.now(),
            params: { name: method, arguments: params }
        });
        return response.data.result;
    } catch (error) {
        console.error(`MCP call failed (${method}):`, error.message);
        return null;
    }
}

// API: Get validator performance history
app.get('/api/validators/history', async (req, res) => {
    const { vote_pubkey, hours = 24 } = req.query;

    let query = `
        SELECT
            timestamp,
            vote_pubkey,
            activated_stake,
            commission,
            skip_rate,
            is_delinquent
        FROM default.validator_performance
        WHERE timestamp >= now() - INTERVAL ${parseInt(hours)} HOUR
    `;

    if (vote_pubkey) {
        query += ` AND vote_pubkey = '${vote_pubkey}'`;
    }

    query += ' ORDER BY timestamp DESC LIMIT 1000';

    try {
        const data = await queryClickHouse(query);
        res.json(data);
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// API: Get network metrics history
app.get('/api/network/history', async (req, res) => {
    const { hours = 24 } = req.query;

    const query = `
        SELECT
            timestamp,
            slot,
            tps,
            active_validators,
            delinquent_validators,
            total_stake
        FROM default.network_metrics
        WHERE timestamp >= now() - INTERVAL ${parseInt(hours)} HOUR
        ORDER BY timestamp DESC
        LIMIT 1000
    `;

    try {
        const data = await queryClickHouse(query);
        res.json(data);
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// API: Get top validators by average performance
app.get('/api/validators/top', async (req, res) => {
    const { limit = 10, hours = 24 } = req.query;

    const query = `
        SELECT
            vote_pubkey,
            avg(skip_rate) as avg_skip_rate,
            avg(activated_stake) as avg_stake,
            avg(commission) as avg_commission,
            sum(is_delinquent) as times_delinquent,
            count() as sample_count
        FROM default.validator_performance
        WHERE timestamp >= now() - INTERVAL ${parseInt(hours)} HOUR
        GROUP BY vote_pubkey
        HAVING avg_skip_rate < 20 AND times_delinquent = 0
        ORDER BY avg_skip_rate ASC, avg_stake DESC
        LIMIT ${parseInt(limit)}
    `;

    try {
        const data = await queryClickHouse(query);
        res.json(data);
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// API: Get TPS trends
app.get('/api/network/tps-trend', async (req, res) => {
    const { hours = 6 } = req.query;

    const query = `
        SELECT
            toStartOfFiveMinute(timestamp) as time_bucket,
            avg(tps) as avg_tps,
            max(tps) as max_tps,
            min(tps) as min_tps
        FROM default.network_metrics
        WHERE timestamp >= now() - INTERVAL ${parseInt(hours)} HOUR
        GROUP BY time_bucket
        ORDER BY time_bucket ASC
    `;

    try {
        const data = await queryClickHouse(query);
        res.json(data);
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// API: Get validator performance stats
app.get('/api/validators/stats', async (req, res) => {
    const query = `
        SELECT
            count(DISTINCT vote_pubkey) as total_tracked,
            countIf(is_delinquent = 0) as active_count,
            countIf(is_delinquent = 1) as delinquent_count,
            avg(skip_rate) as avg_skip_rate,
            avg(activated_stake) as avg_stake
        FROM (
            SELECT
                vote_pubkey,
                argMax(is_delinquent, timestamp) as is_delinquent,
                argMax(skip_rate, timestamp) as skip_rate,
                argMax(activated_stake, timestamp) as activated_stake
            FROM default.validator_performance
            WHERE timestamp >= now() - INTERVAL 1 HOUR
            GROUP BY vote_pubkey
        )
    `;

    try {
        const data = await queryClickHouse(query);
        res.json(data);
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Health check
app.get('/health', (req, res) => {
    res.json({ status: 'ok', service: 'solana-clickhouse-api' });
});

// Start server
app.listen(PORT, () => {
    console.log(`Solana ClickHouse API running on port ${PORT}`);
    console.log(`ClickHouse: ${CLICKHOUSE_URL}`);
    console.log(`MCP Server: ${MCP_URL}`);
});

module.exports = app;
