# NAIRI eBPF Runtime Assets

This directory contains runtime-analysis assets used by the backend pipeline.

## Layout

1. `probes/`
   1. `bpftrace` probe scripts (`trace_fs.bt`, `trace_net.bt`, `trace_properties.bt`, `trace_runtime.bt`).
2. `parsers/`
   1. Log parsers and export tooling (`parse_trace_experiment_csv.py`).
3. `runners/`
   1. Runner wrappers intended for production orchestration integration.
4. `fixtures/`
   1. Test fixtures for parser and probe-contract regression tests.

## Notes

1. Manual experiment orchestration script currently lives at:
   1. `research/trace-experiments/run_trace_experiments.sh`
2. Probe contracts are referenced in:
   1. `spec/nairi-runtime-analysis.md`
