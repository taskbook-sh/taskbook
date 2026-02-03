# Kubernetes Deployment

This guide covers deploying the taskbook server to Kubernetes.

## Prerequisites

- Kubernetes cluster (1.19+)
- kubectl configured
- Container registry access
- PostgreSQL database (managed service or self-hosted)

## Container Image

Build and push the server image:

```bash
# Build
docker build -f Dockerfile.server -t your-registry/taskbook-server:latest .

# Push
docker push your-registry/taskbook-server:latest
```

## Kubernetes Manifests

### Namespace

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: taskbook
```

### Secret

Store database credentials securely:

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: taskbook-db-credentials
  namespace: taskbook
type: Opaque
stringData:
  username: taskbook
  password: your-secure-password
```

### ConfigMap

Non-sensitive configuration:

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: taskbook-config
  namespace: taskbook
data:
  TB_HOST: "0.0.0.0"
  TB_PORT: "8080"
  TB_DB_HOST: "postgres.database.svc.cluster.local"
  TB_DB_PORT: "5432"
  TB_DB_NAME: "taskbook"
  TB_SESSION_EXPIRY_DAYS: "30"
  RUST_LOG: "info"
```

### Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: taskbook-server
  namespace: taskbook
  labels:
    app: taskbook-server
spec:
  replicas: 2
  selector:
    matchLabels:
      app: taskbook-server
  template:
    metadata:
      labels:
        app: taskbook-server
    spec:
      containers:
        - name: taskbook-server
          image: your-registry/taskbook-server:latest
          ports:
            - containerPort: 8080
              name: http
          env:
            - name: TB_HOST
              valueFrom:
                configMapKeyRef:
                  name: taskbook-config
                  key: TB_HOST
            - name: TB_PORT
              valueFrom:
                configMapKeyRef:
                  name: taskbook-config
                  key: TB_PORT
            - name: TB_DB_HOST
              valueFrom:
                configMapKeyRef:
                  name: taskbook-config
                  key: TB_DB_HOST
            - name: TB_DB_PORT
              valueFrom:
                configMapKeyRef:
                  name: taskbook-config
                  key: TB_DB_PORT
            - name: TB_DB_NAME
              valueFrom:
                configMapKeyRef:
                  name: taskbook-config
                  key: TB_DB_NAME
            - name: TB_DB_USER
              valueFrom:
                secretKeyRef:
                  name: taskbook-db-credentials
                  key: username
            - name: TB_DB_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: taskbook-db-credentials
                  key: password
            - name: TB_SESSION_EXPIRY_DAYS
              valueFrom:
                configMapKeyRef:
                  name: taskbook-config
                  key: TB_SESSION_EXPIRY_DAYS
            - name: RUST_LOG
              valueFrom:
                configMapKeyRef:
                  name: taskbook-config
                  key: RUST_LOG
          resources:
            requests:
              memory: "64Mi"
              cpu: "100m"
            limits:
              memory: "256Mi"
              cpu: "500m"
          livenessProbe:
            httpGet:
              path: /api/v1/health
              port: http
            initialDelaySeconds: 5
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /api/v1/health
              port: http
            initialDelaySeconds: 5
            periodSeconds: 5
```

### Service

```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: taskbook-server
  namespace: taskbook
spec:
  selector:
    app: taskbook-server
  ports:
    - port: 80
      targetPort: 8080
      name: http
  type: ClusterIP
```

### Ingress

Using nginx ingress controller:

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: taskbook-server
  namespace: taskbook
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
    - hosts:
        - taskbook.example.com
      secretName: taskbook-tls
  rules:
    - host: taskbook.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: taskbook-server
                port:
                  number: 80
```

## Deploy

```bash
# Create namespace
kubectl apply -f namespace.yaml

# Create secret (edit with your credentials first)
kubectl apply -f secret.yaml

# Create config
kubectl apply -f configmap.yaml

# Deploy
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f ingress.yaml

# Check status
kubectl -n taskbook get pods
kubectl -n taskbook logs -l app=taskbook-server
```

## PostgreSQL Options

### Managed Database Services

For production, consider using a managed PostgreSQL service:

- **AWS**: Amazon RDS for PostgreSQL
- **GCP**: Cloud SQL for PostgreSQL
- **Azure**: Azure Database for PostgreSQL
- **DigitalOcean**: Managed Databases

Update `TB_DB_HOST` in the ConfigMap to point to your managed database endpoint.

### In-Cluster PostgreSQL

For testing or small deployments, you can run PostgreSQL in the cluster:

```yaml
# postgres.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: taskbook
spec:
  serviceName: postgres
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
        - name: postgres
          image: postgres:16-alpine
          ports:
            - containerPort: 5432
          env:
            - name: POSTGRES_USER
              valueFrom:
                secretKeyRef:
                  name: taskbook-db-credentials
                  key: username
            - name: POSTGRES_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: taskbook-db-credentials
                  key: password
            - name: POSTGRES_DB
              value: taskbook
          volumeMounts:
            - name: postgres-data
              mountPath: /var/lib/postgresql/data
  volumeClaimTemplates:
    - metadata:
        name: postgres-data
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 10Gi
---
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: taskbook
spec:
  selector:
    app: postgres
  ports:
    - port: 5432
  clusterIP: None
```

Update ConfigMap:
```yaml
TB_DB_HOST: "postgres.taskbook.svc.cluster.local"
```

## Helm Chart

For easier deployment, you can create a Helm chart. Basic structure:

```
taskbook-server/
├── Chart.yaml
├── values.yaml
└── templates/
    ├── deployment.yaml
    ├── service.yaml
    ├── ingress.yaml
    ├── configmap.yaml
    └── secret.yaml
```

### values.yaml

```yaml
replicaCount: 2

image:
  repository: your-registry/taskbook-server
  tag: latest
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 80

ingress:
  enabled: true
  className: nginx
  host: taskbook.example.com
  tls: true
  clusterIssuer: letsencrypt-prod

database:
  host: postgres.database.svc.cluster.local
  port: 5432
  name: taskbook
  existingSecret: taskbook-db-credentials
  secretUsernameKey: username
  secretPasswordKey: password

config:
  sessionExpiryDays: 30
  logLevel: info

resources:
  requests:
    memory: 64Mi
    cpu: 100m
  limits:
    memory: 256Mi
    cpu: 500m
```

## Monitoring

### Prometheus Metrics

The server exposes health endpoints that can be scraped:

```yaml
# ServiceMonitor for Prometheus Operator
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: taskbook-server
  namespace: taskbook
spec:
  selector:
    matchLabels:
      app: taskbook-server
  endpoints:
    - port: http
      path: /api/v1/health
      interval: 30s
```

### Logging

Logs are written to stdout in JSON format when `RUST_LOG` is set. They can be collected by:

- Fluentd/Fluent Bit
- Loki
- CloudWatch Logs (EKS)
- Cloud Logging (GKE)

## Scaling

### Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: taskbook-server
  namespace: taskbook
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: taskbook-server
  minReplicas: 2
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
```

### Database Connection Pooling

For high-scale deployments, consider using PgBouncer:

```yaml
# Add PgBouncer sidecar or deployment
# Update TB_DB_HOST to point to PgBouncer
```

## Security

### Network Policies

Restrict traffic to only necessary connections:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: taskbook-server
  namespace: taskbook
spec:
  podSelector:
    matchLabels:
      app: taskbook-server
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              name: ingress-nginx
      ports:
        - port: 8080
  egress:
    - to:
        - namespaceSelector:
            matchLabels:
              name: taskbook
          podSelector:
            matchLabels:
              app: postgres
      ports:
        - port: 5432
    - to:
        - namespaceSelector: {}
          podSelector:
            matchLabels:
              k8s-app: kube-dns
      ports:
        - port: 53
          protocol: UDP
```

### Pod Security

```yaml
# Add to deployment spec
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  fsGroup: 1000

containers:
  - name: taskbook-server
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop:
          - ALL
```

## Troubleshooting

### Check Pod Status

```bash
kubectl -n taskbook get pods
kubectl -n taskbook describe pod taskbook-server-xxx
```

### View Logs

```bash
kubectl -n taskbook logs -l app=taskbook-server --tail=100
kubectl -n taskbook logs -l app=taskbook-server -f  # Follow
```

### Test Database Connection

```bash
kubectl -n taskbook run -it --rm debug --image=postgres:16-alpine -- \
  psql "postgres://user:pass@postgres:5432/taskbook"
```

### Test Health Endpoint

```bash
kubectl -n taskbook port-forward svc/taskbook-server 8080:80 &
curl http://localhost:8080/api/v1/health
```
