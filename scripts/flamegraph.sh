#!/bin/bash

ROOCH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

PR_NUMBER=$1

if [[ "$(uname)" != "Linux" ]]; then
  echo "run flamegraph only in linux. exit"
fi

echo "PR_NUMBER $PR_NUMBER"

cmd="RUST_LOG=error cargo bench --bench bench_tx_exec -- --profile-time=10"
echo "run flamegraph with cmd: ${cmd}"
eval "$cmd"


aws s3api put-object --bucket flamegraph.rooch.network --key "$PR_NUMBER"/"bench_tx_exec.svg" --body $ROOCH_DIR/target/criterion/bench_tx_exec/profile/flamegraph.svg
aws s3api put-object --bucket flamegraph.rooch.network --key "$PR_NUMBER"/"bench_tx_exec.svg" --body $ROOCH_DIR/target/criterion/bench_tx_exec/profile/profile.pb
