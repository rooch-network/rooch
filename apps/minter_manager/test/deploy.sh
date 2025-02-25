#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

# Export your address here
export PACKAGE_ADDRESS="0x701c21bf1c8cd5af8c42983890d8ca55e7a820171b8e744c13f2d9998bf76cc3"

# Deploy minter_manager package
cd minter_manager
rooch move publish --named-addresses minter_manager="$PACKAGE_ADDRESS",app_admin="$PACKAGE_ADDRESS"