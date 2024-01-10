#!/bin/env bash

# Build the app in profiling mode
cargo build --profile profiling

# Generate perf.data and flamechart for the app
# cargo install flamegraph
flamegraph --flamechart -F 2000 -- ./target/profiling/anime-games-launcher

# Format perf.data to the profiler.firefox.com compatible format
perf script -F +pid > perf-output.data
