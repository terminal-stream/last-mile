# TSLM Helm Chart

A Helm chart for deploying TSLM (Terminal Stream Last Mile) WebSocket Gateway on Kubernetes.

## Prerequisites

- Kubernetes 1.19+
- Helm 3.2.0+

## Installing the Chart

### From OCI Registry (Recommended)

```bash
helm install my-tslm oci://ghcr.io/terminal-stream/charts/last-mile --version 0.1.0
```

### From Source

```bash
git clone https://github.com/terminal-stream/last-mile.git
cd last-mile
helm install my-tslm charts/last-mile
```

## Uninstalling the Chart

```bash
helm uninstall my-tslm
```

## Configuration

The following table lists the configurable parameters of the TSLM chart and their default values.

### Basic Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `1` |
| `image.repository` | Image repository | `ghcr.io/terminal-stream/last-mile` |
| `image.pullPolicy` | Image pull policy | `IfNotPresent` |
| `image.tag` | Image tag (defaults to chart appVersion) | `""` |

### Service Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `service.type` | Service type (ClusterIP, NodePort, LoadBalancer) | `ClusterIP` |
| `service.public.port` | Public listener service port | `8080` |
| `service.private.port` | Private listener service port | `8081` |

### TSLM Server Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `config.public.enabled` | Enable public listener | `true` |
| `config.public.port` | Public listener port | `8080` |
| `config.public.defaultEndpointPermissions` | Default permissions for public connections | `["Subscribe"]` |
| `config.private.enabled` | Enable private listener | `true` |
| `config.private.port` | Private listener port | `8081` |
| `config.private.defaultEndpointPermissions` | Default permissions for private connections | `["CreateChannel", "NotifyChannel"]` |
| `config.env.RUST_LOG` | Logging level | `"info"` |

### Resource Limits

| Parameter | Description | Default |
|-----------|-------------|---------|
| `resources.limits.cpu` | CPU limit | `1000m` |
| `resources.limits.memory` | Memory limit | `512Mi` |
| `resources.requests.cpu` | CPU request | `100m` |
| `resources.requests.memory` | Memory request | `128Mi` |

### Autoscaling

| Parameter | Description | Default |
|-----------|-------------|---------|
| `autoscaling.enabled` | Enable HPA | `false` |
| `autoscaling.minReplicas` | Minimum replicas | `1` |
| `autoscaling.maxReplicas` | Maximum replicas | `10` |
| `autoscaling.targetCPUUtilizationPercentage` | Target CPU utilization | `80` |

## Examples

### Expose with LoadBalancer

```bash
helm install my-tslm oci://ghcr.io/terminal-stream/charts/last-mile \
  --set service.type=LoadBalancer
```

### Enable Authentication

```bash
helm install my-tslm oci://ghcr.io/terminal-stream/charts/last-mile \
  --set config.public.authTokens="{token1,token2}"
```

### Set Resource Limits

```bash
helm install my-tslm oci://ghcr.io/terminal-stream/charts/last-mile \
  --set resources.limits.cpu=2000m \
  --set resources.limits.memory=1Gi
```

### Enable Rate Limiting

```bash
helm install my-tslm oci://ghcr.io/terminal-stream/charts/last-mile \
  --set config.public.rateLimitPerSecond=10 \
  --set config.public.maxConnections=1000
```

### Custom Values File

Create a `values.yaml` file:

```yaml
service:
  type: LoadBalancer

config:
  public:
    authTokens:
      - "my-secure-token"
    maxConnections: 5000
    rateLimitPerSecond: 50
  env:
    RUST_LOG: "debug"

resources:
  limits:
    cpu: 2000m
    memory: 1Gi
  requests:
    cpu: 200m
    memory: 256Mi

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 20
  targetCPUUtilizationPercentage: 70
```

Install with custom values:

```bash
helm install my-tslm oci://ghcr.io/terminal-stream/charts/last-mile -f values.yaml
```

## Testing the Deployment

After installation, test the deployment:

```bash
# Get the service endpoint
export SERVICE_IP=$(kubectl get svc my-tslm-last-mile -o jsonpath='{.status.loadBalancer.ingress[0].ip}')

# Download the TSLM client
curl -sSL https://raw.githubusercontent.com/terminal-stream/last-mile/main/scripts/install.sh | bash

# Run a test
tslm-client --url ws://$SERVICE_IP:8080 test
```

## Security Considerations

- Use authentication tokens for production deployments
- Enable NetworkPolicy to restrict traffic
- Use TLS/SSL with Ingress for production
- Set appropriate resource limits
- Enable PodDisruptionBudget for high availability

## Support

- GitHub: https://github.com/terminal-stream/last-mile
- Issues: https://github.com/terminal-stream/last-mile/issues
