# Deployment Guide: solahaha.com

This guide covers deploying the Solana MCP Server to solahaha.com.

## Quick Start

### Option 1: Automated GitHub Actions Deployment (Recommended)

1. **Add GitHub Secrets:**
   - Go to: https://github.com/openSVM/solana-mcp-server/settings/secrets/actions
   - Add secret `DEPLOY_SSH_KEY` with the SSH private key for solahaha.com
   - Optionally add `DEPLOY_SSH_USER` (defaults to 'larp')

2. **Trigger Deployment:**
   - Go to: https://github.com/openSVM/solana-mcp-server/actions/workflows/deploy-prod.yml
   - Click "Run workflow"
   - Select "production" environment
   - Click "Run workflow"

3. **Verify:**
   ```bash
   curl https://solahaha.com/health
   # Expected: {"status":"ok","version":"1.1.1",...}
   ```

### Option 2: Manual Deployment

**Deployment package location:** `/tmp/solahaha-deploy/solana-mcp-server-deploy.tar.gz` (12 MB)

**Steps:**

1. **Upload package to server:**
   ```bash
   scp /tmp/solahaha-deploy/solana-mcp-server-deploy.tar.gz solahaha.com:/tmp/
   ```

2. **SSH to server and deploy:**
   ```bash
   ssh solahaha.com
   cd /tmp
   tar -xzf solana-mcp-server-deploy.tar.gz
   bash deploy.sh
   ```

3. **Verify deployment:**
   ```bash
   # On server
   sudo systemctl status solana-mcp-server
   curl http://localhost:3001/health

   # From outside
   curl https://solahaha.com/health
   ```

---

## What Gets Deployed

### Binary
- **Location:** `/opt/solana-mcp-server/solana-mcp-server`
- **Size:** ~30 MB (optimized release build)
- **Version:** 1.1.1
- **Features:** Full sBPF integration with liteSVM

### Configuration
- **Location:** `/opt/solana-mcp-server/config.json`
- **RPC URL:** https://api.mainnet-beta.solana.com
- **Commitment:** confirmed
- **Cache:** Enabled

### Systemd Service
- **Service:** `solana-mcp-server.service`
- **User:** www-data
- **Port:** 3001 (internal)
- **Auto-restart:** Enabled
- **Logs:** `journalctl -u solana-mcp-server -f`

### Nginx Configuration
- **Config:** `/etc/nginx/sites-available/solahaha.com`
- **Domain:** solahaha.com
- **SSL:** HTTPS (Let's Encrypt)
- **Endpoints:**
  - `GET /health` - Health check (public)
  - `POST /api/mcp` - MCP JSON-RPC API (public)
  - `GET /metrics` - Prometheus metrics (localhost only)
  - `GET /` - Static files (existing site)

---

## Architecture

```
Internet
   ↓
HTTPS (443) → Nginx
   ↓
Reverse Proxy
   ↓
localhost:3001 → Solana MCP Server
   ↓
Solana RPC (api.mainnet-beta.solana.com)
```

---

## Available Endpoints

### 1. Health Check
```bash
curl https://solahaha.com/health
```
**Response:**
```json
{
  "status": "ok",
  "version": "1.1.1",
  "protocol": "2025-06-18",
  "service": "solana-mcp-server",
  "capabilities": {
    "tools": true,
    "resources": true,
    "prompts": false,
    "sampling": false
  }
}
```

### 2. MCP API
```bash
curl -X POST https://solahaha.com/api/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "id": 1
  }'
```

### 3. sBPF Tools
All three new sBPF tools are available:

**Validate Binary:**
```bash
curl -X POST https://solahaha.com/api/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "validateSbpfBinary",
      "arguments": {
        "programBinary": "<base64-encoded-elf>"
      }
    },
    "id": 1
  }'
```

**Deploy Program Locally:**
```bash
curl -X POST https://solahaha.com/api/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "deploySbpfProgramLocal",
      "arguments": {
        "programBinary": "<base64-encoded-elf>"
      }
    },
    "id": 1
  }'
```

**Test Program:**
```bash
curl -X POST https://solahaha.com/api/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "testSbpfProgram",
      "arguments": {
        "programBinary": "<base64-encoded-elf>",
        "accounts": [...],
        "instructionData": "<base64>"
      }
    },
    "id": 1
  }'
```

---

## Management Commands

### Service Management
```bash
# Check status
sudo systemctl status solana-mcp-server

# View logs
sudo journalctl -u solana-mcp-server -f

# Restart service
sudo systemctl restart solana-mcp-server

# Stop service
sudo systemctl stop solana-mcp-server

# Start service
sudo systemctl start solana-mcp-server
```

### Nginx Management
```bash
# Test configuration
sudo nginx -t

# Reload configuration
sudo systemctl reload nginx

# View error logs
sudo tail -f /var/log/nginx/error.log

# View access logs
sudo tail -f /var/log/nginx/access.log
```

### Server Monitoring
```bash
# Check if service is listening
sudo lsof -i :3001

# Test local health
curl http://localhost:3001/health

# Test external health
curl https://solahaha.com/health

# View metrics (from server)
curl http://localhost:3001/metrics
```

---

## Troubleshooting

### Service Won't Start
```bash
# Check logs
sudo journalctl -u solana-mcp-server -n 50

# Check permissions
ls -la /opt/solana-mcp-server/

# Check port availability
sudo lsof -i :3001

# Test binary manually
cd /opt/solana-mcp-server
sudo -u www-data ./solana-mcp-server web --port 3001
```

### Nginx Errors
```bash
# Check nginx config
sudo nginx -t

# Check site is enabled
ls -la /etc/nginx/sites-enabled/solahaha.com

# Check nginx status
sudo systemctl status nginx

# View errors
sudo tail -f /var/log/nginx/error.log
```

### SSL Certificate Issues
```bash
# Check certificate
sudo certbot certificates

# Renew certificate
sudo certbot renew

# Test SSL
curl -v https://solahaha.com/health 2>&1 | grep SSL
```

---

## Rollback

If issues occur after deployment:

1. **Stop new service:**
   ```bash
   sudo systemctl stop solana-mcp-server
   ```

2. **Restore previous nginx config:**
   ```bash
   sudo cp /etc/nginx/sites-available/solahaha.com.backup \
           /etc/nginx/sites-available/solahaha.com
   sudo nginx -t && sudo systemctl reload nginx
   ```

3. **Remove deployment:**
   ```bash
   sudo rm -rf /opt/solana-mcp-server
   sudo rm /etc/systemd/system/solana-mcp-server.service
   sudo systemctl daemon-reload
   ```

---

## Security Notes

1. **Firewall:** Ensure ports 80 and 443 are open
2. **SSL:** HTTPS is required for production
3. **Metrics:** Only accessible from localhost
4. **User:** Service runs as www-data (non-root)
5. **Updates:** Keep Rust dependencies updated (`cargo audit`)

---

## Performance

### Expected Metrics
- **Binary validation:** <10ms
- **VM deployment:** <100ms
- **Program execution:** <200ms
- **RPC queries:** 50-500ms (depends on Solana RPC)
- **Memory usage:** 50-100MB baseline
- **CPU usage:** <5% idle, <50% under load

### Monitoring
```bash
# Check resource usage
sudo systemctl status solana-mcp-server

# Detailed metrics
curl http://localhost:3001/metrics
```

---

## Updating

### Manual Update
1. Build new binary:
   ```bash
   cd /home/larp/openSVM/solana-mcp-server
   cargo build --release
   ```

2. Create new deployment package:
   ```bash
   bash scripts/create-deploy-package.sh
   ```

3. Deploy:
   ```bash
   scp /tmp/solahaha-deploy/solana-mcp-server-deploy.tar.gz solahaha.com:/tmp/
   ssh solahaha.com "cd /tmp && tar -xzf solana-mcp-server-deploy.tar.gz && bash deploy.sh"
   ```

### Automated Update (GitHub Actions)
Just push to main branch or manually trigger the workflow.

---

## Support

- **Logs:** `sudo journalctl -u solana-mcp-server -f`
- **GitHub Issues:** https://github.com/openSVM/solana-mcp-server/issues
- **Documentation:** See PRODUCTION_VALIDATION.md for test results

---

**Deployment Status:** Ready to deploy ✅
**Package Location:** `/tmp/solahaha-deploy/solana-mcp-server-deploy.tar.gz`
**Package Size:** 12 MB
**Version:** 1.1.1
**Date:** 2026-01-08
