// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { getRoochNodeUrl } from '@roochnetwork/rooch-sdk';
import { createNetworkConfig } from '@roochnetwork/rooch-sdk-kit';

const { networkConfig, useNetworkVariable, useNetworkVariables } = createNetworkConfig({
  mainnet: {
    url: getRoochNodeUrl('mainnet'),
    variables: {
      portal:'https://localhost:8083/session/',
      counterContract: '0xf859b4113ddd951a694e2d5d3f5849e1ccd43b3cfef92ec8f8f8a46200d3df75'
    },
  },
  testnet: {
    url: getRoochNodeUrl('testnet'),
    variables: {
      portal:'https://localhost:8083/session/',
      counterContract: '0xf859b4113ddd951a694e2d5d3f5849e1ccd43b3cfef92ec8f8f8a46200d3df75'
    },
  },
  localnet: {
    url: getRoochNodeUrl('localnet'),
    variables: {
      portal:'https://localhost:8083/session/',
      counterContract: '0xf859b4113ddd951a694e2d5d3f5849e1ccd43b3cfef92ec8f8f8a46200d3df75'
    },
  },
});

export { networkConfig, useNetworkVariable, useNetworkVariables };
