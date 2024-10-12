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
      btcGasAddress: 'bc1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4sugkfjt'
    },
  },
  testnet: {
    url: getRoochNodeUrl('testnet'),
    variables: {
      roochOperatingAddress: ROOCH_NFT_OPERATING_ADDRESS,
      mintAddress: ROOCH_MINT_OPERATING_ADDRESS,
      btcGasAddress: 'tb1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4stqqxgy'
    },
  },
  localnet: {
    url: getRoochNodeUrl('localnet'),
    variables: {
      roochOperatingAddress: ROOCH_NFT_OPERATING_ADDRESS,
      mintAddress: ROOCH_MINT_OPERATING_ADDRESS,
      btcGasAddress: 'tb1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4stqqxgy'
    },
  },
});

export { networkConfig, useNetworkVariable, useNetworkVariables };
