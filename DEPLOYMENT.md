# TSLM Deployment Guide

This guide explains how to deploy TSLM (Terminal Stream Last Mile) using Helm on Kubernetes and test it with the official client.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Deployment Options](#deployment-options)
- [Client Installation](#client-installation)
- [Testing](#testing)
- [Production Configuration](#production-configuration)
- [Troubleshooting](#troubleshooting)

## Prerequisites

- Kubernetes cluster (1.19+)
  - Local: minikube, kind, k3s, Docker Desktop
  - Cloud: GKE, EKS, AKS, or any managed Kubernetes
- Helm 3.2.0+
- `kubectl` configured for your cluster

## Quick Start

### 1. Install TSLM Server on Kubernetes

```bash
# Install from OCI registry (recommended)
helm install my-tslm oci://ghcr.io/terminal-stream/charts/last-mile --version 0.1.0

# Or from source
git clone https://github.com/terminal-stream/last-mile.git
cd last-mile
helm install my-tslm charts/last-mile
```

### 2. Verify Installation

```bash
# Check pod status
kubectl get pods -l app.kubernetes.io/name=last-mile

# Check service
kubectl get svc -l app.kubernetes.io/name=last-mile

# View logs
kubectl logs -l app.kubernetes.io/name=last-mile -f
```

### 3. Expose the Service

**Option A: Port Forward (local testing)**
```bash
kubectl port-forward svc/my-tslm-last-mile 8080:8080 8081:8081
```

**Option B: LoadBalancer (cloud)**
```bash
helm upgrade my-tslm oci://ghcr.io/terminal-stream/charts/last-mile \
  --reuse-values \
  --set service.type=LoadBalancer
```

**Option C: NodePort (local cluster)**
```bash
helm upgrade my-tslm oci://ghcr.io/terminal-stream/charts/last-mile \
  --reuse-values \
  --set service.type=NodePort \
  --set service.public.nodePort=30080
```

### 4. Install TSLM Client

```bash
curl -sSL https://raw.githubusercontent.com/terminal-stream/last-mile/main/scripts/install.sh | bash
```

Or download manually:
```bash
# Linux AMD64
curl -L https://github.com/terminal-stream/last-mile/releases/latest/download/tslm-client-linux-amd64 -o tslm-client
chmod +x tslm-client
sudo mv tslm-client /usr/local/bin/

# macOS ARM64 (Apple Silicon)
curl -L https://github.com/terminal-stream/last-mile/releases/latest/download/tslm-client-darwin-arm64 -o tslm-client
chmod +x tslm-client
sudo mv tslm-client /usr/local/bin/
```

### 5. Test the Deployment

```bash
# Using port-forward
tslm-client --url ws://localhost:8080 test

# Using LoadBalancer
export SERVICE_IP=$(kubectl get svc my-tslm-last-mile -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
tslm-client --url ws://$SERVICE_IP:8080 test

# Using NodePort
export NODE_IP=$(kubectl get nodes -o jsonpath='{.items[0].status.addresses[0].address}')
tslm-client --url ws://$NODE_IP:30080 test
```

## Deployment Options

### Minikube

```bash
minikube start
helm install my-tslm charts/last-mile
minikube service my-tslm-last-mile --url
# Use the returned URL with tslm-client
```

### Kind (Kubernetes in Docker)

```bash
kind create cluster
helm install my-tslm charts/last-mile

# Port forward to access
kubectl port-forward svc/my-tslm-last-mile 8080:8080
tslm-client --url ws://localhost:8080 test
```

### GKE (Google Kubernetes Engine)

```bash
# Create cluster
gcloud container clusters create tslm-cluster --num-nodes=3

# Install with LoadBalancer
helm install my-tslm oci://ghcr.io/terminal-stream/charts/last-mile \
  --set service.type=LoadBalancer \
  --set config.public.maxConnections=10000 \
  --set resources.limits.cpu=2000m \
  --set resources.limits.memory=2Gi

# Get external IP
kubectl get svc my-tslm-last-mile

# Test
tslm-client --url ws://<EXTERNAL-IP>:8080 test
```

### EKS (Amazon Elastic Kubernetes Service)

```bash
# Create cluster
eksctl create cluster --name tslm-cluster --nodes 3

# Install with LoadBalancer
helm install my-tslm oci://ghcr.io/terminal-stream/charts/last-mile \
  --set service.type=LoadBalancer

# Get hostname
kubectl get svc my-tslm-last-mile -o jsonpath='{.status.loadBalancer.ingress[0].hostname}'

# Test
tslm-client --url ws://<HOSTNAME>:8080 test
```

## Client Installation

### Install Script (Recommended)

```bash
curl -sSL https://raw.githubusercontent.com/terminal-stream/last-mile/main/scripts/install.sh | bash
```

The script will:
- Detect your OS and architecture
- Download the appropriate binary
- Verify checksums
- Install to `~/.local/bin/tslm-client`
- Show usage instructions

### Manual Installation

Download from [GitHub Releases](https://github.com/terminal-stream/last-mile/releases):

Available platforms:
- `tslm-client-linux-amd64`
- `tslm-client-linux-arm64`
- `tslm-client-darwin-amd64` (macOS Intel)
- `tslm-client-darwin-arm64` (macOS Apple Silicon)

```bash
# Example for Linux AMD64
wget https://github.com/terminal-stream/last-mile/releases/latest/download/tslm-client-linux-amd64
chmod +x tslm-client-linux-amd64
sudo mv tslm-client-linux-amd64 /usr/local/bin/tslm-client
```

### Custom Install Location

```bash
export INSTALL_DIR=/opt/bin
curl -sSL https://raw.githubusercontent.com/terminal-stream/last-mile/main/scripts/install.sh | bash
```

## Testing

### Quick Test

```bash
# Run built-in test scenario
tslm-client --url ws://localhost:8080 test
```

### Manual Testing

```bash
# Terminal 1: Subscribe to a channel
tslm-client --url ws://localhost:8080 subscribe my-channel

# Terminal 2: Create channel and publish (requires private listener)
tslm-client --url ws://localhost:8081 create-channel my-channel
tslm-client --url ws://localhost:8081 publish my-channel "Hello, World!"
```

### Continuous Publishing

```bash
# Publish 100 messages with 500ms interval
tslm-client --url ws://localhost:8081 publish test-channel "Test message" --count 100 --interval 500
```

## Production Configuration

### High Availability

```yaml
# values-ha.yaml
replicaCount: 3

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 20
  targetCPUUtilizationPercentage: 70

podDisruptionBudget:
  enabled: true
  minAvailable: 2

resources:
  limits:
    cpu: 2000m
    memory: 2Gi
  requests:
    cpu: 500m
    memory: 512Mi

affinity:
  podAntiAffinity:
    preferredDuringSchedulingIgnoredDuringExecution:
      - weight: 100
        podAffinityTerm:
          labelSelector:
            matchLabels:
              app.kubernetes.io/name: last-mile
          topologyKey: kubernetes.io/hostname
```

```bash
helm install my-tslm oci://ghcr.io/terminal-stream/charts/last-mile -f values-ha.yaml
```

### Security Configuration

```yaml
# values-secure.yaml
config:
  public:
    authTokens:
      - "secure-public-token-123"
    maxConnections: 5000
    rateLimitPerSecond: 50
    maxMessageSize: 65536

  private:
    authTokens:
      - "secure-internal-token-456"
    maxConnections: 100
    maxMessageSize: 131072

networkPolicy:
  enabled: true
  policyTypes:
    - Ingress
  ingress:
    # Allow public listener from anywhere
    - from: []
      ports:
        - protocol: TCP
          port: 8080
    # Restrict private listener to same namespace
    - from:
        - podSelector: {}
      ports:
        - protocol: TCP
          port: 8081

ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: tslm.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: tslm-tls
      hosts:
        - tslm.example.com
```

### Resource Limits by Use Case

**Small (< 100 connections):**
```yaml
resources:
  limits:
    cpu: 500m
    memory: 256Mi
  requests:
    cpu: 100m
    memory: 128Mi
```

**Medium (100-1000 connections):**
```yaml
resources:
  limits:
    cpu: 1000m
    memory: 512Mi
  requests:
    cpu: 250m
    memory: 256Mi
```

**Large (1000+ connections):**
```yaml
resources:
  limits:
    cpu: 4000m
    memory: 4Gi
  requests:
    cpu: 1000m
    memory: 1Gi

autoscaling:
  enabled: true
  minReplicas: 5
  maxReplicas: 50
```

## Troubleshooting

### Pod Not Starting

```bash
# Check pod events
kubectl describe pod -l app.kubernetes.io/name=last-mile

# Check logs
kubectl logs -l app.kubernetes.io/name=last-mile --tail=100

# Common issues:
# - ConfigMap not created: Check helm template output
# - Image pull errors: Verify registry access
# - Resource limits: Check node resources
```

### Connection Refused

```bash
# Verify service
kubectl get svc my-tslm-last-mile

# Test from inside cluster
kubectl run -it --rm debug --image=busybox --restart=Never -- sh
# Inside pod:
# telnet my-tslm-last-mile 8080

# Check if ports are listening
kubectl exec -it <pod-name> -- netstat -tlnp
```

### Authentication Errors

```bash
# Check ConfigMap
kubectl get configmap my-tslm-last-mile-config -o yaml

# Verify auth tokens are set correctly
# Client must send token in Sec-WebSocket-Protocol header
```

### Performance Issues

```bash
# Check resource usage
kubectl top pods -l app.kubernetes.io/name=last-mile

# View metrics
kubectl get hpa

# Increase resources
helm upgrade my-tslm oci://ghcr.io/terminal-stream/charts/last-mile \
  --reuse-values \
  --set resources.limits.cpu=4000m \
  --set resources.limits.memory=4Gi
```

### Helm Chart Issues

```bash
# Validate chart
helm lint charts/last-mile

# Render templates to check output
helm template my-tslm charts/last-mile

# Check release history
helm history my-tslm

# Rollback if needed
helm rollback my-tslm
```

## Uninstalling

```bash
# Uninstall Helm release
helm uninstall my-tslm

# Verify cleanup
kubectl get all -l app.kubernetes.io/name=last-mile

# Remove PVCs if any
kubectl delete pvc -l app.kubernetes.io/name=last-mile
```

## Next Steps

- [Read the Architecture Guide](CLAUDE.md)
- [Review Configuration Options](charts/last-mile/README.md)
- [Report Issues](https://github.com/terminal-stream/last-mile/issues)
- [View Examples](client/examples/)

## Support

- GitHub: https://github.com/terminal-stream/last-mile
- Issues: https://github.com/terminal-stream/last-mile/issues
- Discussions: https://github.com/terminal-stream/last-mile/discussions
