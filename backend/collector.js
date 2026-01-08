const axios = require('axios');

const CLICKHOUSE_URL = 'http://localhost:8123';
const MCP_URL = 'http://localhost:3000/api/mcp';
const COLLECTION_INTERVAL = 60000; // 1 minute

let lastSlot = 0;

// Helper: Insert into ClickHouse
async function insertClickHouse(table, columns, values) {
    const query = `INSERT INTO default.${table} (${columns.join(', ')}) VALUES ${values}`;
    try {
        await axios.post(CLICKHOUSE_URL, query);
        console.log(`✓ Inserted into ${table}`);
    } catch (error) {
        console.error(`✗ Failed to insert into ${table}:`, error.message);
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

// Calculate skip rate from epoch credits
function calculateSkipRate(epochCredits) {
    if (!epochCredits || epochCredits.length === 0) return 0;

    const recent = epochCredits.slice(-2);
    const avgCredits = recent.reduce((sum, c) => sum + c[1], 0) / recent.length;
    const expectedCredits = 400000;

    return Math.max(0, Math.min(100, ((expectedCredits - avgCredits) / expectedCredits) * 100));
}

// Collect validator performance data
async function collectValidatorData() {
    console.log('[Validator Collector] Starting collection...');

    const voteAccounts = await callMCP('getVoteAccounts');
    if (!voteAccounts || !voteAccounts.current) {
        console.error('[Validator Collector] Failed to fetch vote accounts');
        return;
    }

    const timestamp = Math.floor(Date.now() / 1000);
    const values = [];

    // Active validators
    for (const v of voteAccounts.current) {
        const skipRate = calculateSkipRate(v.epochCredits);
        const epochCredits = v.epochCredits && v.epochCredits.length > 0
            ? v.epochCredits[v.epochCredits.length - 1][1]
            : 0;

        values.push(`(
            ${timestamp},
            '${v.votePubkey}',
            '${v.nodePubkey || ''}',
            ${v.activatedStake},
            ${v.commission},
            ${epochCredits},
            ${v.lastVote || 0},
            ${v.rootSlot || 0},
            ${skipRate.toFixed(2)},
            0,
            ''
        )`);
    }

    // Delinquent validators
    if (voteAccounts.delinquent) {
        for (const v of voteAccounts.delinquent.slice(0, 50)) {
            values.push(`(
                ${timestamp},
                '${v.votePubkey}',
                '${v.nodePubkey || ''}',
                ${v.activatedStake},
                ${v.commission},
                0,
                ${v.lastVote || 0},
                ${v.rootSlot || 0},
                100,
                1,
                ''
            )`);
        }
    }

    if (values.length > 0) {
        await insertClickHouse(
            'validator_performance',
            ['timestamp', 'vote_pubkey', 'node_pubkey', 'activated_stake', 'commission',
             'epoch_credits', 'last_vote', 'root_slot', 'skip_rate', 'is_delinquent', 'version'],
            values.join(',')
        );
    }

    console.log(`[Validator Collector] Collected ${values.length} validators`);
}

// Collect network metrics
async function collectNetworkMetrics() {
    console.log('[Network Collector] Starting collection...');

    const [slotData, voteAccounts] = await Promise.all([
        callMCP('getSlot'),
        callMCP('getVoteAccounts')
    ]);

    if (!slotData || !slotData.slot) {
        console.error('[Network Collector] Failed to fetch slot');
        return;
    }

    const currentSlot = slotData.slot;
    const timestamp = Math.floor(Date.now() / 1000);

    // Calculate TPS
    let tps = 0;
    if (lastSlot > 0) {
        const slotDiff = currentSlot - lastSlot;
        const timeDiff = COLLECTION_INTERVAL / 1000;
        tps = Math.round((slotDiff * 500) / timeDiff); // ~500 tx per slot estimate
    }
    lastSlot = currentSlot;

    const activeValidators = voteAccounts?.current?.length || 0;
    const delinquentValidators = voteAccounts?.delinquent?.length || 0;
    const totalStake = voteAccounts?.current?.reduce((sum, v) => sum + v.activatedStake, 0) || 0;

    const values = `(
        ${timestamp},
        ${currentSlot},
        0,
        ${tps},
        0,
        ${activeValidators + delinquentValidators},
        ${activeValidators},
        ${delinquentValidators},
        ${totalStake}
    )`;

    await insertClickHouse(
        'network_metrics',
        ['timestamp', 'slot', 'block_height', 'tps', 'transaction_count',
         'total_validators', 'active_validators', 'delinquent_validators', 'total_stake'],
        values
    );

    console.log(`[Network Collector] Slot: ${currentSlot}, TPS: ${tps}, Validators: ${activeValidators}/${delinquentValidators}`);
}

// Main collection loop
async function collect() {
    try {
        await Promise.all([
            collectValidatorData(),
            collectNetworkMetrics()
        ]);
    } catch (error) {
        console.error('[Collector] Error:', error.message);
    }
}

// Start collector
console.log('='.repeat(60));
console.log('Solana Data Collector for ClickHouse');
console.log('='.repeat(60));
console.log(`MCP Server: ${MCP_URL}`);
console.log(`ClickHouse: ${CLICKHOUSE_URL}`);
console.log(`Collection Interval: ${COLLECTION_INTERVAL / 1000}s`);
console.log('='.repeat(60));

// Initial collection
collect();

// Periodic collection
setInterval(collect, COLLECTION_INTERVAL);

// Graceful shutdown
process.on('SIGINT', () => {
    console.log('\n[Collector] Shutting down gracefully...');
    process.exit(0);
});
