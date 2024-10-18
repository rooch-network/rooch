// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { getRoochNodeUrl } from '@roochnetwork/rooch-sdk';
import { createNetworkConfig } from '@roochnetwork/rooch-sdk-kit';

import { FAUCET_MAINNET, FAUCET_TESTNET, ROOCH_NFT_OPERATING_ADDRESS, ROOCH_MINT_OPERATING_ADDRESS } from 'src/config/constant';

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
      btcGasAddress: 'bc1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4sugkfjt',
      gasMarketAddress: '0x701c21bf1c8cd5af8c42983890d8ca55e7a820171b8e744c13f2d9998bf76cc3',
      faucetUrl: FAUCET_MAINNET,
      faucetAddress: '0x701c21bf1c8cd5af8c42983890d8ca55e7a820171b8e744c13f2d9998bf76cc3',
      faucetObject: '0xd5723eda84f691ae2623da79312c7909b1737c5b3866ecc5dbd6aa21718ff15d',
      BTCMemPool: 'https://mempool.space/tx/',
    },
  },
  testnet: {
    url: getRoochNodeUrl('testnet'),
    variables: {
      roochOperatingAddress: ROOCH_NFT_OPERATING_ADDRESS,
      mintAddress: ROOCH_MINT_OPERATING_ADDRESS,
      btcGasAddress: 'tb1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4stqqxgy',
      gasMarketAddress: '0x872502737008ac71c4c008bb3846a688bfd9fa54c6724089ea51b72f813dc71e',
      faucetUrl: FAUCET_TESTNET,
      faucetAddress: '0x701c21bf1c8cd5af8c42983890d8ca55e7a820171b8e744c13f2d9998bf76cc3',
      faucetObject: '0xd5723eda84f691ae2623da79312c7909b1737c5b3866ecc5dbd6aa21718ff15d',
      BTCMemPool: 'https://mempool.space/testnet/tx/',
    },
  },
  localnet: {
    url: getRoochNodeUrl('localnet'),
    variables: {
      roochOperatingAddress: ROOCH_NFT_OPERATING_ADDRESS,
      mintAddress: ROOCH_MINT_OPERATING_ADDRESS,
      btcGasAddress: 'tb1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4stqqxgy',
      gasMarketAddress: '0x872502737008ac71c4c008bb3846a688bfd9fa54c6724089ea51b72f813dc71e',
      faucetUrl: FAUCET_TESTNET,
      faucetAddress: '0x701c21bf1c8cd5af8c42983890d8ca55e7a820171b8e744c13f2d9998bf76cc3',
      faucetObject: '0xd5723eda84f691ae2623da79312c7909b1737c5b3866ecc5dbd6aa21718ff15d',
      BTCMemPool: 'https://mempool.space/testnet/tx/',
    },
  },
});

export { networkConfig, useNetworkVariable, useNetworkVariables };
