// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { getRoochNodeUrl } from '@roochnetwork/rooch-sdk';
import { createNetworkConfig } from '@roochnetwork/rooch-sdk-kit';

import { ROOCH_NFT_OPERATING_ADDRESS, ROOCH_MINT_OPERATING_ADDRESS } from 'src/config/constant';

console.log(
  '🚀 ~ file: use-networks.ts:8 ~ ROOCH_NFT_OPERATING_ADDRESS, ROOCH_MINT_OPERATING_ADDRESS:',
  ROOCH_NFT_OPERATING_ADDRESS,
  ROOCH_MINT_OPERATING_ADDRESS
);

const { networkConfig, useNetworkVariable, useNetworkVariables } = createNetworkConfig({
  mainnet: {
    url: getRoochNodeUrl('mainnet'),
    variables: {
      roochOperatingAddress: ROOCH_NFT_OPERATING_ADDRESS,
      mintAddress: ROOCH_MINT_OPERATING_ADDRESS,
    },
  },
  testnet: {
    url: getRoochNodeUrl('testnet'),
    variables: {
      roochOperatingAddress: ROOCH_NFT_OPERATING_ADDRESS,
      mintAddress: ROOCH_MINT_OPERATING_ADDRESS,
    },
  },
  localnet: {
    url: getRoochNodeUrl('localnet'),
    variables: {
      roochOperatingAddress: ROOCH_NFT_OPERATING_ADDRESS,
      mintAddress: ROOCH_MINT_OPERATING_ADDRESS,
    },
  },
});

export { networkConfig, useNetworkVariable, useNetworkVariables };
