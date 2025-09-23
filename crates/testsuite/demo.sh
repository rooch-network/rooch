#!/bin/bash

# Rooch Enhanced Testsuite Framework Demo
# This script demonstrates the new debugging capabilities

echo "🧪 Rooch Enhanced Testsuite Framework - Demo Script"
echo "================================================="
echo

echo "📋 Available Environment Variables:"
echo "  ROOCH_TEST_LOG_LEVEL=minimal|normal|verbose|debug"
echo "  ROOCH_TEST_TEMPLATE_DEBUG=true|false"
echo "  ROOCH_TEST_SHOW_PROGRESS=true|false"
echo "  ROOCH_TEST_TIMEOUT=<seconds>"
echo "  ROOCH_TEST_HELP=true"
echo

echo "🎯 Demo 1: Show help information"
echo "ROOCH_TEST_HELP=true cargo check -p testsuite"
echo

echo "🎯 Demo 2: Development mode with progress"
echo "ROOCH_TEST_LOG_LEVEL=verbose ROOCH_TEST_SHOW_PROGRESS=true cargo test --test integration -- --nocapture"
echo

echo "🎯 Demo 3: Full debugging mode"
echo "ROOCH_TEST_LOG_LEVEL=debug ROOCH_TEST_TEMPLATE_DEBUG=true ROOCH_TEST_SHOW_PROGRESS=true cargo test --test integration -- --nocapture"
echo

echo "🎯 Demo 4: CI/minimal mode"
echo "ROOCH_TEST_LOG_LEVEL=minimal cargo test --test integration"
echo

echo "📖 For detailed documentation:"
echo "  - crates/testsuite/DEBUGGING.md"
echo "  - crates/testsuite/features/README.md"
echo

echo "✨ Key Features:"
echo "  ✅ Enhanced error messages with context"
echo "  ✅ Template variable debugging"
echo "  ✅ Progressive test execution indicators"
echo "  ✅ Configurable logging levels"
echo "  ✅ Detailed assertion failure messages"
echo "  ✅ Command execution history tracking"
echo "  ✅ 100% backward compatible"
echo

echo "🚀 Example Error Message Improvements:"
echo
echo "Before:"
echo "  ERROR integration: run_cli cmd: payment-channel fail: String(\"Transaction error...\")"
echo "  thread 'test' panicked at crates/testsuite/tests/integration.rs:554:5:"
echo "  splited_args should not empty, the orginal_args:\\"
echo
echo "After:"
echo "  🧪 [16/42] payment_channel_operations - Running command: payment-channel withdraw-revenue"
echo "    ❌ payment-channel withdraw-revenue --owner rooch1abc... --amount 5000"
echo "       Error: Transaction execution failed: MoveAbort { location: 0x3::payment_revenue, abort_code: 1 }"
echo "       Template vars used: [\"$.did[1].did_address\"]"
echo "       Available context keys: [\"account\", \"did\", \"payment-channel\", \"address_mapping\"]"
echo
echo "  ❌ Assertion failed:"
echo "     Expression: executed == moveabort"
echo "     Expected: executed"
echo "     Actual: moveabort"
echo "     Operator: =="
echo "     Template vars used: [\"$.payment-channel[-1].execution_info.status.type\"]"
echo

echo "🎉 Ready to use! Try the commands above to experience the enhanced debugging capabilities."