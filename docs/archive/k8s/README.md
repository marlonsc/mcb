# Archived Kubernetes Artifacts

This directory preserves inactive Kubernetes artifacts removed from the active
`k8s/` tree.

## Files

- `legacy-manifests.bak/` - archived legacy manifests and deploy script. They
  are historical context only and must not be used for deployment.

The archived set is superseded because it used stale image tags, old
configuration shape, placeholder secret manifests, metrics paths not present in
the current server, and imperative `kubectl apply` deployment guidance.
