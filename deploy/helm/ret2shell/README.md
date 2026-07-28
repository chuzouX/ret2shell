# Rhythm Arena Helm Chart

This chart installs Rhythm Arena into the fixed `ret2shell-platform` namespace.
The internal release and resource names retain the `ret2shell` prefix.

Important constraints:

- Install with `-n ret2shell-platform --create-namespace`.
- The release namespace must be `ret2shell-platform`.
- The platform is a singleton workload and must not be scaled above one replica.

Quick start:

```bash
helm install ret2shell ./deploy/helm/ret2shell -n ret2shell-platform --create-namespace
```

Useful switches:

- `platform.exposure.type=ingress|nodePort`
- `postgresql.mode=internal|external`
- `valkey.mode=internal|external`
- `valkey.architecture=standalone|replication`
- `nats.mode=internal|external`
- `nats.replicaCount=<n>`
- `victoriaLogs.mode=disabled|internal|external`

The bundled services expose pod annotations, labels, priority classes,
topology-spread constraints, disruption budgets, and metrics settings.

Example renders:

```bash
helm template ret2shell ./deploy/helm/ret2shell -n ret2shell-platform -f ./deploy/helm/ret2shell/examples/values-ingress-internal.yaml
helm template ret2shell ./deploy/helm/ret2shell -n ret2shell-platform -f ./deploy/helm/ret2shell/examples/values-nodeport-external.yaml
```

Before production use, replace the platform image, signing key, external
domain, and all default passwords and tokens.
