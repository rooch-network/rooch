// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { getRoochNodeUrl } from '@roochnetwork/rooch-sdk';
import { createNetworkConfig } from '@roochnetwork/rooch-sdk-kit';

const { networkConfig, useNetworkVariable, useNetworkVariables } = createNetworkConfig({
  mainnet: {
    url: getRoochNodeUrl('mainnet'),
    variables: {
    },
  },
  testnet: {
    url: getRoochNodeUrl('testnet'),
    variables: {
    },
  },
  localnet: {
    url: getRoochNodeUrl('localnet'),
    variables: {
    },
  },
});

export { networkConfig, useNetworkVariable, useNetworkVariables };
