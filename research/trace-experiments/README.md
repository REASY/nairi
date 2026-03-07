# Trace Experiments

This folder contains manual/experimental runtime tracing workflows.

## Runner

1. `run_trace_experiments.sh`
   1. Runs two-phase trace capture (`fresh_launch`, `second_launch`) against an Android target.
   2. Uses probes from `backend/runtime/ebpf/probes` by default.
   3. Supports overriding probe location via `--probes-dir`.

## Output Post-Processing

1. Use parser at:
   1. `backend/runtime/ebpf/parsers/parse_trace_experiment_csv.py`
