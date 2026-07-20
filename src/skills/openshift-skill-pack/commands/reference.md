# OpenShift Command Reference Reference

## Command Categories

| Category | Risk Level | Examples |
|----------|-----------|----------|
| Status/Info | Low | get, describe, top |
| Diagnostic | Low | logs, inspect, troubleshoot |
| Configuration | Medium | set image, set env, set resources |
| Action | Medium | delete, restart, scale |
| Admin | High-Critical | adm drain, adm upgrade, taint |

## Usage Guidelines

### Always Use
- `-n <namespace>` to specify namespace
- `-o wide` for additional columns
- `--sort-by='.lastTimestamp'` for chronological events
- `--no-pager` for verbose output

### Verify Before Acting
- Check events first
- Check describe output
- Check logs
- Consider cascade effects
- Document findings

## References

- [OpenShift CLI Documentation](https://docs.openshift.com/container-platform/latest/cli_reference/openshift_cli/index.html)
- [Kubernetes CLI Reference](https://kubernetes.io/docs/reference/kubectl/)