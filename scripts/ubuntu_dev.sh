#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0
# This script sets up the environment for installing necessary dependencies.
#
# Usage ./ubuntu_dev.sh

if [ "$(whoami)" != 'root' ]; then
  sudo apt update
  sudo apt install git curl clang lld pkg-config libssl-dev libsqlite3-dev
else
  apt update
  apt install git curl clang lld pkg-config libssl-dev libsqlite3-dev
fi

cat << EOF

=== Congratulations! ===
You have installed the required system dependencies,
now ready to install Rust, if you already have Rust
installed you can choose to cancel (Option 3).

EOF

echo "Installing Rust ..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
