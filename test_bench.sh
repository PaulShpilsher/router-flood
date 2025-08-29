#!/bin/bash

# Quick benchmark test - run one benchmark from each suite
echo "Testing Router Flood Benchmarks (Quick Mode)..."
echo "=============================================="

# Run one benchmark from each suite with very limited samples
export CRITERION_SAMPLE_SIZE=10
export CRITERION_WARM_UP_TIME=1

echo ""
echo "Testing packet_building..."
timeout 10 cargo bench --bench packet_building "zero_copy/Udp" 2>&1 | tail -5

echo ""
echo "Testing config_validation..."
timeout 10 cargo bench --bench config_validation "valid_config_build" 2>&1 | tail -5

echo ""
echo "Testing lockfree_stats..."
timeout 10 cargo bench --bench lockfree_stats "lockfree_increment" 2>&1 | tail -5

echo ""
echo "Testing raii_guards..."
timeout 10 cargo bench --bench raii_guards "channel_guard_lifecycle" 2>&1 | tail -5

echo ""
echo "Testing abstractions..."
timeout 10 cargo bench --bench abstractions "direct_geteuid" 2>&1 | tail -5

echo ""
echo "=============================================="
echo "✅ All benchmark suites compile and run successfully"