#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

# A script to check whether a local commit related to Move repo is ready for a PR.

BASE=$(git rev-parse --show-toplevel)

set -e
CARGO_HOME=${CARGO_HOME:-~/.cargo}

echo $CARGO_HOME/cargo-nextest

if [ ! -f ${CARGO_HOME}/bin/cargo-nextest ];then
  echo "install nextest"
  if [[ "$(uname)" == "Linux" ]]; then
    curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
  elif [[ "$(uname)" == "Darwin" ]]; then
    curl -LsSf https://get.nexte.st/latest/mac | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
  fi
fi

# Run only tests which would also be run on CI
export ENV_TEST_ON_CI=1

while getopts "hcxtdgma" opt; do
  case $opt in
    h)
      cat <<EOF
Usage:
    check_pr <flags>
Flags:
    -h   Print this help
    -c   Check the core prover crates using cargo xfmt/xclippy.
         This is the default if no flags are provided.
    -x   Like -c, but adds more crates (specifically all which depend
         on move-model)
    -t   In addition to xfmt/xclippy, run cargo test
    -d   Run documentation generation, abi generation, etc. for move-stdlib
         and other tested frameworks.
    -g   Run the Move git checks script (whitespace check). This works
         only for committed clients.
    -m   Run the Move unit and verification tests.
    -a   Run all of the above
EOF
      exit 1
      ;;
    c)
      CHECK=1
      ;;
    x)
      CHECK=1
      CHECK_MORE=1
      ;;
    d)
      GEN_ARTIFACTS=1
      ;;
    g)
      GIT_CHECKS=1
      ;;
    t)
      CHECK=1
      ALSO_TEST=1
      ;;
    m)
      MOVE_TESTS=1
      ;;
    a)
      CHECK=1
      CHECK_MORE=1
      GEN_ARTIFACTS=1
      GIT_CHECKS=1
      ALSO_TEST=1
      MOVE_TESTS=1
      MOVE_E2E_TESTS=1
      ;;
  esac
done

MOVE_TEST_CRATES="\
  moveos/moveos-stdlib/moveos-stdlib\
  crates/rooch-framework\
"

if [ ! -z "$CHECK" ]; then
  cargo fmt -- --check
  cargo clippy --all-targets --all-features --tests --benches -- -D warnings
fi

cargo build

if [ ! -z "$ALSO_TEST" ]; then
    cargo nextest run --workspace --all-features
    cargo test -p testsuite --test integration
fi

if [ ! -z "$MOVE_TESTS" ]; then
  for crate in $MOVE_TEST_CRATES; do
    echo "*************** [check-pr] Move tests $crate"
    (
      cargo run --bin rooch move test -p $crate
    )
  done
fi

