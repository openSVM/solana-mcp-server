# Kubernetes Deployment with Autoscaling

This directory contains Kubernetes manifests for deploying the Solana MCP Server with dynamic autoscaling capabilities.

## Prerequisites

- Kubernetes cluster with Metrics Server installed
- Prometheus Operator (for custom metrics)
- Prometheus Adapter (for custom metrics in HPA)

## Quick Deployment

```bash
# Deploy the application
kubectl apply -f deployment.yaml

# Deploy autoscaling configuration
kubectl apply -f hpa.yaml

# Check deployment status
kubectl get pods,svc,hpa -l app=solana-mcp-server
```

## Files

### deployment.yaml
- **Deployment**: Main application with resource requests/limits
- **Service**: ClusterIP service exposing metrics port
- **ServiceMonitor**: Prometheus monitoring configuration

### hpa.yaml
- **HorizontalPodAutoscaler**: Autoscaling configuration
- **ConfigMap**: Prometheus Adapter configuration for custom metrics

## Autoscaling Configuration

### Resource-based scaling:
- **CPU**: Scale when average CPU > 70%
- **Memory**: Scale when average memory > 80%

### Custom metrics-based scaling:
- **Request rate**: Scale when RPS > 100 per pod
- **Request latency**: Scale when P95 latency > 500ms

### Scaling behavior:
- **Min replicas**: 1
- **Max replicas**: 10
- **Scale up**: Fast (up to 100% increase every 15s)
- **Scale down**: Conservative (max 10% decrease every 60s)

## Monitoring

### Metrics exposed at `/metrics`:
- `solana_mcp_rpc_requests_total` - Total RPC requests
- `solana_mcp_rpc_requests_successful_total` - Successful requests
- `solana_mcp_rpc_requests_failed_total` - Failed requests  
- `solana_mcp_rpc_request_duration_seconds` - Request duration histogram
- `solana_mcp_rpc_errors_total` - Errors by type

### Health check available at `/health`

## Custom Metrics Setup

For custom metrics scaling to work, you need:

1. **Prometheus Operator** installed in your cluster
2. **Prometheus Adapter** configured with the ConfigMap from hpa.yaml

```bash
# Install Prometheus Operator (if not already installed)
kubectl apply -f https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/bundle.yaml

# Install Prometheus Adapter
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm install prometheus-adapter prometheus-community/prometheus-adapter \
  --set prometheus.url=http://prometheus-operated.monitoring.svc:9090 \
  --set-file rules.custom=./hpa.yaml
```

## Scaling Test

To test autoscaling behavior:

```bash
# Generate load to trigger CPU-based scaling
kubectl run -i --tty load-generator --rm --image=busybox --restart=Never -- /bin/sh
# Inside the pod:
while true; do wget -q -O- http://solana-mcp-service:8080/health; done

# Monitor scaling
kubectl get hpa solana-mcp-server-hpa --watch

# Check pod scaling
kubectl get pods -l app=solana-mcp-server --watch
```

## Resource Requirements

### Per Pod:
- **CPU**: 250m request, 500m limit
- **Memory**: 512Mi request, 1Gi limit

### Cluster Resources (at max scale):
- **CPU**: 2.5 cores request, 5 cores limit  
- **Memory**: 5Gi request, 10Gi limit

## Security

The deployment includes security hardening:
- Non-root user execution
- Capability dropping
- Security context configuration
- Network policies (add as needed)

## Troubleshooting

### Check HPA status:
```bash
kubectl describe hpa solana-mcp-server-hpa
```

### Check metrics availability:
```bash
kubectl get --raw "/apis/metrics.k8s.io/v1beta1/pods" | jq .
kubectl get --raw "/apis/custom.metrics.k8s.io/v1beta1" | jq .
```

### Check pod metrics:
```bash
kubectl port-forward svc/solana-mcp-service 8080:8080
curl http://localhost:8080/metrics
```

### Common issues:
1. **Metrics Server not running**: Install with `kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml`
2. **Custom metrics unavailable**: Ensure Prometheus Adapter is properly configured and can reach Prometheus
3. **Pods not scaling**: Check resource requests are set and HPA is not in error state