# SSH Tunnel for Runtime Analysis

Use this directory for host-side SSH tunnel config to reach remote Redroid.

## Files

1. `ssh.config.example`: tracked template.
2. `ssh.config`: local real config (ignored by git).
3. `tunnel.sock`: local SSH control socket created by scripts.

## Workflow

1. Copy template:

```bash
cp ops/ssh/ssh.config.example ops/ssh/ssh.config
```

2. Edit `ops/ssh/ssh.config` with real host/user/key details.

3. Start tunnel:

```bash
make runtime-tunnel-up
```

4. Build/use runtime image:

```bash
make build-runtime
```

5. Run runtime analysis container with `ADB_CONNECTION_STRING=host.docker.internal:5555`.

6. Stop tunnel when done:

```bash
make runtime-tunnel-down
```
