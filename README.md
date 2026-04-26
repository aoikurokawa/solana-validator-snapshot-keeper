# Solana validator snapshot keeper

A Rust port of [SOL-Strategies/solana-validator-snapshot-keeper](https://github.com/SOL-Strategies/solana-validator-snapshot-keeper).

Keeps fresh Solana snapshots on disk: discovers snapshot-serving nodes on the cluster, pulls the latest snapshot via parallel range-segmented HTTP downloads, prunes older ones, and stays out of the way when the local validator is actively voting.

> **Status:** phase 1 — CLI, YAML config, validation, and lock-file plumbing are in place. The discovery / download / pruning / hooks pipeline is stubbed and will land in subsequent phases.

## Build

```
cargo build --release
```

The resulting binary is at `target/release/solana-validator-snapshot-keeper`.

## Usage

```
solana-validator-snapshot-keeper [OPTIONS] <COMMAND>

Commands:
  run   Run the snapshot keeper (once or on an interval)

Options:
  -c, --config <CONFIG>         Path to config file
                                [default: ~/solana-validator-snapshot-keeper/config.yml]
      --log-level <LOG_LEVEL>   Override log level (debug, info, warn, error)
      --log-disable-timestamps  Disable timestamps in log output
```

### One-shot run

```
solana-validator-snapshot-keeper --config /etc/svsk/config.yml run
```

### Interval mode

Runs aligned to a midnight + N·interval grid (matches the Go reference):

```
solana-validator-snapshot-keeper --config /etc/svsk/config.yml run --on-interval 4h
```

## Configuration

YAML config compatible with the upstream Go tool. All fields below show their defaults.

```yaml
log:
  level: info              # debug | info | warn | error
  format: text             # text | json | logfmt
  disable_timestamps: false

validator:
  rpc_url: "http://127.0.0.1:8899"
  active_identity_pubkey: ""    # required

cluster:
  name: mainnet-beta            # mainnet-beta | testnet
  rpc_url: ""                   # auto-derived from cluster name when empty

snapshots:
  directory: /mnt/accounts/snapshots   # must exist and be writable
  discovery:
    candidates:
      min_suitable_full: 3
      min_suitable_incremental: 5
      sort_order: latency             # latency | slot_age
    probe:
      concurrency: 500
      max_latency: 100ms
  download:
    min_speed: 60mb                   # accepts b / kb / mb / gb / tb
    min_speed_check_delay: 7s
    timeout: 30m
    connections: 8
  age:
    remote:
      max_slots: 1300
    local:
      max_incremental_slots: 1300

# hooks:
#   on_success:
#     - name: notify-slack
#       cmd: /usr/local/bin/slack-notify.sh
#       args: ["success", "Downloaded snapshot slot {{ SnapshotSlot }} from {{ SourceNode }}"]
#       allow_failure: true
#       stream_output: false
#   on_failure:
#     - name: notify-slack
#       cmd: /usr/local/bin/slack-notify.sh
#       args: ["failure", "Snapshot download failed: {{ Error }}"]
#       allow_failure: true
#       stream_output: false
```

`validator.active_identity_pubkey` is required: the keeper checks whether the local validator currently holds this identity and skips work to avoid impacting an actively voting node.

## Lock file

While running, the keeper writes a JSON lock at `<snapshots.directory>/solana-validator-snapshot-keeper.lock` containing `{ "pid": ..., "started_at": "..." }`. Stale locks (PID no longer alive) are detected via `kill(pid, 0)` and overwritten.

## Differences from the Go reference

- YAML loader: `serde_yaml_ng` (the original `serde_yaml` is archived). Schema is unchanged.
- Logging: built on `tracing`. There is no separate `fatal` level — `fatal` in config is accepted and mapped to `error`.

## Roadmap

- [x] Phase 1: CLI, config, validation, lock file, interval scheduler
- [ ] Phase 2: Solana JSON-RPC client + concurrent node discovery
- [ ] Phase 3: parallel segmented downloader with resume + min-speed check
- [ ] Phase 4: pruner + templated hooks
- [ ] Phase 5: full keeper orchestration (validator-aware abort, freshness skip, etc.)

## License

MIT.
