#!/bin/bash
set -e

echo "📊 Running DuckTape Benchmarks"

# Function to format benchmark results
format_results() {
    local name=$1
    local result=$2
    printf "%-30s %s\n" "$name" "$result"
}

echo "Setting up benchmark environment..."
export RUST_LOG=error
export BENCHMARK_MODE=true

# Run Criterion benchmarks
echo "Running Criterion benchmarks..."
cargo bench

# Custom benchmarks for specific features
echo -e "\nRunning custom feature benchmarks..."

# Command parsing benchmark
echo "Command parsing performance:"
RUST_LOG=error cargo run --release -- \
    --bench-commands ./benches/data/commands.txt \
    2>/dev/null

# Calendar operation benchmark
echo -e "\nCalendar operations performance:"
RUST_LOG=error cargo run --release -- \
    --bench-calendar ./benches/data/calendar_ops.txt \
    2>/dev/null

# WebSocket performance
echo -e "\nWebSocket performance:"
cargo run --release --example websocket_bench -- \
    --connections 100 \
    --duration 30 \
    --rate 100 \
    2>/dev/null

# Memory usage benchmark
echo -e "\nMemory usage benchmark:"
for users in 100 1000 10000; do
    cargo run --release -- --bench-memory-usage $users 2>/dev/null
done

# Load test API endpoints
echo -e "\nAPI endpoint load test:"
if command -v hey >/dev/null 2>&1; then
    # Test API endpoints under load
    hey -n 10000 -c 100 http://localhost:3000/health
    hey -n 1000 -c 50 -m POST -H "Content-Type: application/json" \
        -d '{"command":"list meetings today"}' \
        http://localhost:3000/api/calendar/query
else
    echo "hey load testing tool not found, skipping API load test"
fi

# Generate performance report
echo -e "\nGenerating performance report..."
echo "Performance Report" > bench_results.md
echo "==================" >> bench_results.md
date >> bench_results.md
echo -e "\nSystem Information:" >> bench_results.md
uname -a >> bench_results.md
echo -e "\nRust Version:" >> bench_results.md
rustc --version >> bench_results.md
echo -e "\nBenchmark Results:" >> bench_results.md
cat target/criterion/report/*.json | \
    jq -r '.["benchmark_results"] | to_entries | .[] | "\(.key): \(.value)"' \
    >> bench_results.md 2>/dev/null || true

# Compare with baseline if it exists
if [ -f "bench_baseline.json" ]; then
    echo -e "\nComparing with baseline..."
    cargo bench -- --baseline bench_baseline
fi

# Save new baseline if requested
if [ "$SAVE_BASELINE" = "true" ]; then
    echo "Saving new baseline..."
    cp target/criterion/baseline.json bench_baseline.json
fi

# Check for performance regressions
echo -e "\nChecking for performance regressions..."
if [ -f "bench_baseline.json" ]; then
    regressions=$(cargo bench -- --baseline bench_baseline 2>&1 | grep "Regression" || true)
    if [ ! -z "$regressions" ]; then
        echo "⚠️  Performance regressions detected:"
        echo "$regressions"
        echo "Run with SAVE_BASELINE=true to update baseline if these are expected changes"
    else
        echo "✅ No performance regressions detected"
    fi
fi

# Generate flamegraph if perf is available
if command -v perf >/dev/null 2>&1; then
    echo -e "\nGenerating flamegraph..."
    cargo flamegraph --bench main_benchmark
    mv flamegraph.svg target/flamegraph-$(date +%Y%m%d).svg
fi

echo -e "\n✅ Benchmark run completed"
echo "Results saved to bench_results.md"
if [ -f "target/flamegraph-$(date +%Y%m%d).svg" ]; then
    echo "Flamegraph saved to target/flamegraph-$(date +%Y%m%d).svg"
fi