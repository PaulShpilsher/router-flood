#!/bin/bash

# Run benchmarks with optimized settings for faster execution
echo "Running Router Flood Benchmarks..."
echo "================================"

# Set environment variables for faster benchmarking
export CRITERION_MEASUREMENT_TIME=5
export CRITERION_SAMPLE_SIZE=20

# Run each benchmark suite individually with timeout
benchmarks=(
    "packet_building"
    "config_validation"
    "lockfree_stats"
    "raii_guards"
    "abstractions"
)

for bench in "${benchmarks[@]}"; do
    echo ""
    echo "Running $bench benchmark..."
    echo "----------------------------"
    timeout 60 cargo bench --bench "$bench" 2>&1 | grep -E "(time:|^Benchmarking|error|warning)"
    if [ $? -eq 124 ]; then
        echo "⚠️  $bench benchmark timed out after 60 seconds"
    fi
done

echo ""
echo "================================"
echo "Benchmark suite completed"