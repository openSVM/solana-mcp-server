---
layout: docs
title: "Docker Scaling Guide"
description: "Scaling strategies for Docker environments with autoscaling and orchestration"
order: 12
category: deployment
---

# Docker Scaling Guide

This guide covers scaling strategies for the Solana MCP Server in Docker environments.

## Overview

While Docker doesn't have native autoscaling like Kubernetes, there are several approaches to scale the Solana MCP Server based on demand:

1. **Manual Scaling** - Scale replicas manually using Docker Compose
2. **Docker Swarm Mode** - Built-in orchestration with basic scaling
3. **External Autoscalers** - Third-party tools for dynamic scaling

## Manual Scaling with Docker Compose

### Basic docker-compose.yml

```yaml
version: '3.8'
services:
  solana-mcp-server:
    build: .
    ports:
      - "8080"
    environment:
      - SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
      - SOLANA_COMMITMENT=confirmed
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 30s
    deploy:
      resources:
        limits:
          cpus: '0.5'
          memory: 1G
        reservations:
          cpus: '0.25'
          memory: 512M
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - solana-mcp-server
```

### Scaling Commands

```bash
# Scale to 3 replicas
docker-compose up --scale solana-mcp-server=3 -d

# Scale down to 1 replica
docker-compose up --scale solana-mcp-server=1 -d

# Check current scale
docker-compose ps
```

### Load Balancer Configuration (nginx.conf)

```nginx
events {
    worker_connections 1024;
}

http {
    upstream solana_mcp_backend {
        least_conn;
        server solana-mcp-server_solana-mcp-server_1:8080;
        server solana-mcp-server_solana-mcp-server_2:8080;
        server solana-mcp-server_solana-mcp-server_3:8080;
    }

    server {
        listen 80;
        
        # Health check endpoint
        location /health {
            proxy_pass http://solana_mcp_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
        
        # Metrics endpoint (for monitoring)
        location /metrics {
            proxy_pass http://solana_mcp_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
        
        # Main application (if needed)
        location / {
            proxy_pass http://solana_mcp_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        }
    }
}
```

## Docker Swarm Mode

Docker Swarm provides basic orchestration and scaling capabilities.

### Initialize Swarm

```bash
# Initialize swarm (on manager node)
docker swarm init

# Join worker nodes
docker swarm join --token <token> <manager-ip>:2377
```

### Deploy Stack

Create `docker-stack.yml`:

```yaml
version: '3.8'
services:
  solana-mcp-server:
    image: solana-mcp-server:latest
    ports:
      - target: 8080
        published: 8080
        protocol: tcp
        mode: ingress
    environment:
      - SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
      - SOLANA_COMMITMENT=confirmed
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 30s
    deploy:
      replicas: 2
      resources:
        limits:
          cpus: '0.5'
          memory: 1G
        reservations:
          cpus: '0.25'
          memory: 512M
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
        window: 120s
      update_config:
        parallelism: 1
        delay: 10s
        failure_action: rollback
        monitor: 60s
        max_failure_ratio: 0.3
      rollback_config:
        parallelism: 1
        delay: 5s
        failure_action: pause
        monitor: 60s
        max_failure_ratio: 0.3
    networks:
      - solana-mcp-network

networks:
  solana-mcp-network:
    driver: overlay
```

### Swarm Scaling Commands

```bash
# Deploy the stack
docker stack deploy -c docker-stack.yml solana-mcp

# Scale the service
docker service scale solana-mcp_solana-mcp-server=5

# Check service status
docker service ls
docker service ps solana-mcp_solana-mcp-server

# Update the service
docker service update --image solana-mcp-server:new-version solana-mcp_solana-mcp-server

# Remove the stack
docker stack rm solana-mcp
```

## External Autoscaling Solutions

### 1. Prometheus + Custom Autoscaler

Create a simple autoscaler script:

```bash
#!/bin/bash
# autoscaler.sh - Simple CPU-based autoscaler

STACK_NAME="solana-mcp"
SERVICE_NAME="${STACK_NAME}_solana-mcp-server"
MIN_REPLICAS=1
MAX_REPLICAS=10
CPU_THRESHOLD=70
CHECK_INTERVAL=30

while true; do
    # Get current replicas
    CURRENT_REPLICAS=$(docker service inspect --format='{{.Spec.Mode.Replicated.Replicas}}' $SERVICE_NAME)
    
    # Get average CPU usage (requires monitoring setup)
    CPU_USAGE=$(curl -s http://localhost:9090/api/v1/query?query=avg\(rate\(container_cpu_usage_seconds_total\{name=~\"$SERVICE_NAME.*\"\}\[1m\]\)\)*100 | jq -r '.data.result[0].value[1]' 2>/dev/null || echo "0")
    
    echo "Current replicas: $CURRENT_REPLICAS, CPU usage: $CPU_USAGE%"
    
    # Scale up if CPU is high
    if (( $(echo "$CPU_USAGE > $CPU_THRESHOLD" | bc -l) )) && (( CURRENT_REPLICAS < MAX_REPLICAS )); then
        NEW_REPLICAS=$((CURRENT_REPLICAS + 1))
        echo "Scaling up to $NEW_REPLICAS replicas"
        docker service scale $SERVICE_NAME=$NEW_REPLICAS
    
    # Scale down if CPU is low
    elif (( $(echo "$CPU_USAGE < $((CPU_THRESHOLD - 20))" | bc -l) )) && (( CURRENT_REPLICAS > MIN_REPLICAS )); then
        NEW_REPLICAS=$((CURRENT_REPLICAS - 1))
        echo "Scaling down to $NEW_REPLICAS replicas"
        docker service scale $SERVICE_NAME=$NEW_REPLICAS
    fi
    
    sleep $CHECK_INTERVAL
done
```

Make it executable and run:

```bash
chmod +x autoscaler.sh
./autoscaler.sh &
```

### 2. Using Prometheus and AlertManager

Configure Prometheus rules in `prometheus-rules.yml`:

```yaml
groups:
- name: docker-scaling
  rules:
  - alert: HighCPUUsage
    expr: avg(rate(container_cpu_usage_seconds_total{name=~"solana-mcp.*"}[1m])) * 100 > 70
    for: 2m
    labels:
      severity: warning
      action: scale_up
    annotations:
      summary: "High CPU usage detected"
      description: "CPU usage is above 70% for 2 minutes"
      
  - alert: LowCPUUsage  
    expr: avg(rate(container_cpu_usage_seconds_total{name=~"solana-mcp.*"}[1m])) * 100 < 30
    for: 5m
    labels:
      severity: info
      action: scale_down
    annotations:
      summary: "Low CPU usage detected"
      description: "CPU usage is below 30% for 5 minutes"
```

### 3. Third-party Solutions

#### Orbiter (Docker Swarm Autoscaler)
```bash
# Install Orbiter
docker service create \
  --name orbiter \
  --mount type=bind,source=/var/run/docker.sock,target=/var/run/docker.sock \
  --constraint 'node.role == manager' \
  gianarb/orbiter:latest \
  --metrics-addr=http://prometheus:9090 \
  --log-level=debug
```

## Monitoring and Metrics

### Prometheus Configuration

Add to `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'solana-mcp-server'
    static_configs:
      - targets: ['solana-mcp-server:8080']
    metrics_path: '/metrics'
    scrape_interval: 30s
    
  - job_name: 'docker'
    static_configs:
      - targets: ['docker-exporter:9323']
```

### Key Metrics for Scaling Decisions

1. **CPU Usage**: `rate(container_cpu_usage_seconds_total[1m])`
2. **Memory Usage**: `container_memory_working_set_bytes`
3. **Request Rate**: `rate(solana_mcp_rpc_requests_total[1m])`
4. **Response Time**: `histogram_quantile(0.95, rate(solana_mcp_rpc_request_duration_seconds_bucket[1m]))`
5. **Error Rate**: `rate(solana_mcp_rpc_requests_failed_total[1m])`

### Grafana Dashboard

Import the dashboard configuration from `/docs/grafana-dashboard.json` for visualization.

## Best Practices

1. **Resource Limits**: Always set CPU and memory limits
2. **Health Checks**: Configure proper health checks for containers
3. **Graceful Shutdown**: Ensure containers handle SIGTERM gracefully
4. **Load Balancing**: Use nginx or HAProxy for load balancing
5. **Monitoring**: Monitor key metrics for scaling decisions
6. **Testing**: Test scaling behavior under load

## Load Testing

Use tools like `wrk` or `ab` to test scaling:

```bash
# Install wrk
sudo apt-get install wrk

# Test with increasing load
wrk -t12 -c400 -d30s http://localhost:8080/health

# Monitor scaling during test
watch -n 2 'docker service ps solana-mcp_solana-mcp-server'
```

## Troubleshooting

### Common Issues

1. **Containers not scaling**: Check resource constraints and Docker daemon logs
2. **Load balancer not finding backends**: Verify service discovery and network configuration
3. **High memory usage**: Monitor for memory leaks and adjust limits
4. **Slow scaling**: Adjust check intervals and thresholds

### Debug Commands

```bash
# Check service logs
docker service logs -f solana-mcp_solana-mcp-server

# Inspect service configuration
docker service inspect solana-mcp_solana-mcp-server

# Check node resources
docker node ls
docker node inspect self

# Monitor resource usage
docker stats
```

## Limitations

Unlike Kubernetes HPA, Docker-based scaling has limitations:

1. **No built-in custom metrics support**
2. **Manual configuration required for autoscaling**
3. **Limited to node-level resource monitoring**
4. **No automatic rollback on scaling failures**

For production workloads requiring sophisticated autoscaling, consider using Kubernetes with the HPA configuration provided in `/k8s/hpa.yaml`.