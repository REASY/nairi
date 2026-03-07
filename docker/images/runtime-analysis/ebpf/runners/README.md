# Trace Experiments

This folder contains manual/experimental runtime tracing workflows.

## Runner

1. `run_trace_experiments.sh`
    1. Runs two-phase trace capture (`fresh_launch`, `second_launch`) against an Android target.
    2. Uses probes from `../ebpf/probes` by default.
    3. Supports overriding probe location via `--probes-dir`.
   4. Supports non-interactive execution via `--phase-seconds <N>` as a total time budget across both phases.

## Output Post-Processing

1. Use parser at:
    1. `../parsers/parse_trace_experiment_csv.py`
