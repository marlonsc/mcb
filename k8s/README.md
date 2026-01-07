# 🚀 MCP Context Browser - Kubernetes Deployment

This documentation describes how to deploy MCP Context Browser in a Kubernetes cluster with horizontal auto-scaling using HPA (HorizontalPodAutoscaler).

## 📋 Prerequisites

-   Kubernetes 1.24+
-   Helm 3.x (optional, for dependencies)
-   Cert-Manager (for automatic TLS)
-   NGINX Ingress Controller
-   Prometheus Operator (for metrics and custom HPA)
-   Redis (for distributed cache)
-   PostgreSQL (for metadata)
-   Milvus (for vector store)

## 🏗️ Architecture

```text
Internet → Ingress → Service → Pods (2-10 replicas) → Dependencies
                       ↓
                   HPA (Auto-scaling)
                       ↓
                 Prometheus Metrics
```

### Components

-   **Deployment**: Main application with health checks
-   **HPA**: Auto-scaling based on CPU, memory and custom metrics
-   **Service**: Internal load balancing
-   **Ingress**: External exposure with TLS
-   **ConfigMap**: Application configurations
-   **Secrets**: Sensitive credentials
-   **RBAC**: Access control
-   **NetworkPolicy**: Network security
-   **PodDisruptionBudget**: High availability

## 🚀 Deploy

### 1. Prepare Secrets

Before deployment, you need to create/populate secrets with real values:

```bash
# Example: Encode Redis URL in base64
echo -n "redis://user:password@redis-service:6379/0" | base64

# Update secrets.yaml with encoded values
```

### 2. Deploy Dependencies

```bash
# Redis
helm repo add bitnami https://charts.bitnami.com/bitnami
helm install redis bitnami/redis -n default

# PostgreSQL
helm install postgresql bitnami/postgresql -n default

# Milvus (optional, for advanced vector store)
helm repo add milvus https://milvus-io.github.io/milvus-helm/
helm install milvus milvus/milvus -n default

# Ollama (optional, for local embeddings)
helm repo add ollama https://otwld.github.io/ollama-helm/
helm install ollama ollama-ollama -n default
```

### 3. Deploy Application

```bash
# Complete deploy
./deploy.sh

# Or apply manually
kubectl apply -f . -n default
```

### 4. Verify Deploy

```bash
# Pod status
kubectl get pods -l app=mcp-context-browser

# HPA status
kubectl get hpa mcp-context-browser-hpa

# Application logs
kubectl logs -f deployment/mcp-context-browser

# Metrics
curl http://your-domain.com:3001/api/context/metrics
```

## ⚙️ Configuração

### Auto-scaling

The HPA is configured for:

-   **Minimum**: 2 replicas
-   **Maximum**: 10 replicas
-   **Metrics**:
    -   CPU: 70% average utilization
    -   Memory: 80% average utilization
    -   Requests/s: 100 requests per pod
    -   Active connections: 50 connections per pod

### Resource Limits

```yaml
requests:
  cpu: 500m
  memory: 1Gi
limits:
  cpu: 2000m
  memory: 4Gi
```

### Health Checks

-   **Liveness**: `/api/health` a cada 10s
-   **Readiness**: `/api/health` a cada 5s
-   **Startup**: `/api/health` com timeout de 6 tentativas

## 📊 Monitoramento

### Prometheus Metrics

The ServiceMonitor exposes metrics at `/api/context/metrics`:

-   `mcp_http_requests_total`: Total HTTP requests
-   `mcp_http_request_duration_seconds`: Request duration
-   `mcp_active_connections`: Active connections
-   `mcp_cache_hit_ratio`: Cache hit ratio
-   `mcp_resource_limits_*`: Resource limits

### Dashboards Grafana

Importe o dashboard fornecido em `docs/diagrams/grafana-dashboard.json`.

## 🔧 Troubleshooting

### Problemas Comuns

1.  **Pods don't start**: Check secrets and configmaps
2.  **HPA doesn't scale**: Check Prometheus metrics
3.  **Timeouts**: Ajustar resource limits
4.  **Cache errors**: Verificar conexão Redis

### Debug Commands

```bash
# Ver eventos
kubectl get events --sort-by=.metadata.creationTimestamp

# Descrever recursos
kubectl describe deployment mcp-context-browser
kubectl describe hpa mcp-context-browser-hpa

# Ver logs com contexto
kubectl logs -f deployment/mcp-context-browser --previous

# Port-forward para debug
kubectl port-forward svc/mcp-context-browser-service 3000:80
```

## 🔄 Updates

To update the application:

```bash
# Build new image
docker build -t mcp-context-browser:v0.0.5 .

# Update deployment
kubectl set image deployment/mcp-context-browser mcp-context-browser=mcp-context-browser:v0.0.5

# Rollout
kubectl rollout status deployment/mcp-context-browser
```

## 🛡️ Segurança

-   **RBAC**: ServiceAccount with minimal permissions
-   **NetworkPolicy**: Network traffic control
-   **Secrets**: Base64 encoded credentials
-   **TLS**: Automatic certificates via cert-manager
-   **SecurityContext**: Run as non-root

## 📈 Performance Tuning

### HPA Custom Metrics

For custom metrics, add to HPA:

```yaml
- type: Pods
  pods:
    metric:
      name: mcp_custom_metric
    target:
      type: AverageValue
      averageValue: "100"
```

### Resource Optimization

Ajuste os limites baseado no uso:

```bash
# Monitor resource usage
kubectl top pods -l app=mcp-context-browser

# Adjust limits
kubectl edit deployment mcp-context-browser
```

## 🤝 Suporte

Para issues, consulte:

-   [GitHub Issues](https://github.com/mcp-context-browser/issues)
-   [Documentation](https://docs.mcp-context-browser.com)
-   [Kubernetes Best Practices](https://kubernetes.io/docs/concepts/)
