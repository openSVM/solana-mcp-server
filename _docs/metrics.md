---
layout: docs
title: "Metrics & Monitoring"
description: "Prometheus metrics, Kubernetes autoscaling, and observability setup"
order: 13
category: deployment
---

# Metrics Documentation

The Solana MCP Server exposes comprehensive metrics for monitoring, alerting, and autoscaling. This document describes the available metrics and how to use them.

## Metrics Endpoint

- **URL**: `http://<server>:8080/metrics`
- **Format**: Prometheus text format
- **Content-Type**: `text/plain; version=0.0.4`

## Available Metrics

### RPC Request Metrics

#### `solana_mcp_rpc_requests_total`
- **Type**: Counter
- **Description**: Total number of RPC requests processed
- **Labels**: 
  - `method`: RPC method name (e.g., `getBalance`, `getHealth`)
  - `network`: Network identifier (e.g., `mainnet`, `testnet`)

#### `solana_mcp_rpc_requests_successful_total`
- **Type**: Counter  
- **Description**: Number of successful RPC requests
- **Labels**: 
  - `method`: RPC method name
  - `network`: Network identifier

#### `solana_mcp_rpc_requests_failed_total`
- **Type**: Counter
- **Description**: Number of failed RPC requests
- **Labels**:
  - `method`: RPC method name  
  - `network`: Network identifier
  - `error_type`: Error category (`validation`, `rpc`, `network`, `auth`, `server`)

#### `solana_mcp_rpc_request_duration_seconds`
- **Type**: Histogram
- **Description**: Request duration in seconds
- **Labels**:
  - `method`: RPC method name
  - `network`: Network identifier
- **Buckets**: 0.001, 0.005, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0

#### `solana_mcp_rpc_errors_total`
- **Type**: Counter
- **Description**: Total RPC errors by type and method
- **Labels**:
  - `error_type`: Error category
  - `method`: RPC method name

## Autoscaling Metrics

The following derived metrics are used for Kubernetes HPA:

### `solana_mcp_rpc_requests_per_second`
- **Derived from**: `rate(solana_mcp_rpc_requests_total[1m])`
- **Usage**: Scale up when > 100 RPS per pod
- **Formula**: `rate(solana_mcp_rpc_requests_total{pod="<pod-name>"}[1m])`

### `solana_mcp_rpc_request_duration_seconds_p95`
- **Derived from**: `histogram_quantile(0.95, rate(solana_mcp_rpc_request_duration_seconds_bucket[1m]))`
- **Usage**: Scale up when P95 latency > 500ms
- **Formula**: `histogram_quantile(0.95, rate(solana_mcp_rpc_request_duration_seconds_bucket{pod="<pod-name>"}[1m]))`

## Health Endpoint

- **URL**: `http://<server>:8080/health`
- **Response**: `{"status":"ok","service":"solana-mcp-server"}`
- **Usage**: Kubernetes liveness and readiness probes

## PromQL Query Examples

### Basic Queries

```promql
# Total request rate across all pods
sum(rate(solana_mcp_rpc_requests_total[5m]))

# Error rate by method
sum(rate(solana_mcp_rpc_requests_failed_total[5m])) by (method)

# Average request duration
avg(rate(solana_mcp_rpc_request_duration_seconds_sum[5m]) / rate(solana_mcp_rpc_request_duration_seconds_count[5m]))

# P95 latency
histogram_quantile(0.95, sum(rate(solana_mcp_rpc_request_duration_seconds_bucket[5m])) by (le))
```

### Autoscaling Queries

```promql
# Request rate per pod (used by HPA)
rate(solana_mcp_rpc_requests_total[1m])

# P95 latency per pod (used by HPA)
histogram_quantile(0.95, rate(solana_mcp_rpc_request_duration_seconds_bucket[1m]))

# Error rate percentage
(
  sum(rate(solana_mcp_rpc_requests_failed_total[5m])) /
  sum(rate(solana_mcp_rpc_requests_total[5m]))
) * 100
```

### Resource Utilization

```promql
# CPU utilization per pod
rate(container_cpu_usage_seconds_total{pod=~"solana-mcp-server-.*"}[5m])

# Memory utilization per pod  
container_memory_working_set_bytes{pod=~"solana-mcp-server-.*"}

# Network traffic per pod
rate(container_network_receive_bytes_total{pod=~"solana-mcp-server-.*"}[5m])
rate(container_network_transmit_bytes_total{pod=~"solana-mcp-server-.*"}[5m])
```

## Alerting Rules

### Critical Alerts

```yaml
groups:
- name: solana-mcp-server
  rules:
  # High error rate
  - alert: SolanaMcpHighErrorRate
    expr: |
      (
        sum(rate(solana_mcp_rpc_requests_failed_total[5m])) /
        sum(rate(solana_mcp_rpc_requests_total[5m]))
      ) > 0.1
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "High error rate in Solana MCP Server"
      description: "Error rate is {{ $value | humanizePercentage }} for 2 minutes"

  # High latency
  - alert: SolanaMcpHighLatency
    expr: |
      histogram_quantile(0.95,
        sum(rate(solana_mcp_rpc_request_duration_seconds_bucket[5m])) by (le)
      ) > 2.0
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High latency in Solana MCP Server"
      description: "P95 latency is {{ $value }}s for 5 minutes"

  # No requests (service down)
  - alert: SolanaMcpNoRequests
    expr: |
      sum(rate(solana_mcp_rpc_requests_total[10m])) == 0
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "Solana MCP Server receiving no requests"
      description: "No requests received for 5 minutes"
```

## Grafana Dashboard

### Key Panels

1. **Request Rate**: `sum(rate(solana_mcp_rpc_requests_total[5m])) by (method)`
2. **Error Rate**: `sum(rate(solana_mcp_rpc_requests_failed_total[5m])) by (error_type)`
3. **Latency Percentiles**: 
   - P50: `histogram_quantile(0.50, sum(rate(solana_mcp_rpc_request_duration_seconds_bucket[5m])) by (le))`
   - P95: `histogram_quantile(0.95, sum(rate(solana_mcp_rpc_request_duration_seconds_bucket[5m])) by (le))`
   - P99: `histogram_quantile(0.99, sum(rate(solana_mcp_rpc_request_duration_seconds_bucket[5m])) by (le))`
4. **Pod Count**: `count(up{job="solana-mcp-server"})`
5. **HPA Status**: Custom panel showing current/desired replicas

### Sample Dashboard JSON

```json
{
  "dashboard": {
    "title": "Solana MCP Server",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph", 
        "targets": [
          {
            "expr": "sum(rate(solana_mcp_rpc_requests_total[5m])) by (method)",
            "legendFormat": "{{method}}"
          }
        ]
      },
      {
        "title": "Error Rate %",
        "type": "graph",
        "targets": [
          {
            "expr": "(sum(rate(solana_mcp_rpc_requests_failed_total[5m])) by (method) / sum(rate(solana_mcp_rpc_requests_total[5m])) by (method)) * 100",
            "legendFormat": "{{method}}"
          }
        ]
      }
    ]
  }
}
```

## Integration with Kubernetes HPA

The metrics are designed to work with Kubernetes Horizontal Pod Autoscaler:

1. **Resource Metrics**: CPU and memory utilization (built-in)
2. **Custom Metrics**: Request rate and latency (via Prometheus Adapter)

### HPA Configuration Example

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: solana-mcp-server-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: solana-mcp-server
  minReplicas: 1
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Pods
    pods:
      metric:
        name: solana_mcp_rpc_requests_per_second
      target:
        type: AverageValue
        averageValue: "100"
```

## Troubleshooting

### Metrics Not Available

1. **Check metrics endpoint**:
   ```bash
   curl http://localhost:8080/metrics
   ```

2. **Verify Prometheus scraping**:
   ```bash
   kubectl logs -l app=prometheus
   ```

3. **Check ServiceMonitor**:
   ```bash
   kubectl get servicemonitor solana-mcp-server-monitor -o yaml
   ```

### HPA Not Scaling

1. **Check custom metrics**:
   ```bash
   kubectl get --raw "/apis/custom.metrics.k8s.io/v1beta1/namespaces/default/pods/*/solana_mcp_rpc_requests_per_second"
   ```

2. **Verify Prometheus Adapter**:
   ```bash
   kubectl logs -n monitoring deployment/prometheus-adapter
   ```

3. **Check HPA status**:
   ```bash
   kubectl describe hpa solana-mcp-server-hpa
   ```

## Performance Impact

The metrics collection has minimal performance impact:

- **CPU overhead**: < 1% under normal load
- **Memory overhead**: ~10MB for metrics storage
- **Network overhead**: ~1KB/s for metrics scraping (30s interval)

## Security Considerations

- Metrics endpoint is exposed on all interfaces (0.0.0.0:8080)
- No authentication required for metrics access
- Consider using network policies to restrict access
- Metrics may contain sensitive timing information about operations