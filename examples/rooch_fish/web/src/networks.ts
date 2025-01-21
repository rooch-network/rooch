// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
// Author: Jason Jo

import { getRoochNodeUrl } from "@roochnetwork/rooch-sdk";
import { createNetworkConfig } from "@roochnetwork/rooch-sdk-kit";

const { networkConfig, useNetworkVariable, useNetworkVariables } =
  createNetworkConfig({
    testnet: {
      url: getRoochNodeUrl("testnet"),
    },
    localnet: {
      url: getRoochNodeUrl("localnet"),
    },
  });

export { useNetworkVariable, useNetworkVariables, networkConfig };
