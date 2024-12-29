// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { getRoochNodeUrl } from '@roochnetwork/rooch-sdk';
import { createNetworkConfig } from '@roochnetwork/rooch-sdk-kit';

import { FAUCET_TESTNET, FAUCET_MAINNET, ROOCH_MULTI_SIG_ADDRESS } from 'src/config/constant';



const faucet = (network: 'test' | 'main') => ({
    address: ROOCH_MULTI_SIG_ADDRESS,
    url: network === 'main' ? FAUCET_MAINNET : FAUCET_TESTNET,
    obj: '0xd5723eda84f691ae2623da79312c7909b1737c5b3866ecc5dbd6aa21718ff15d',
  })

const inviter = {
  address: ROOCH_MULTI_SIG_ADDRESS,
  module: 'invitation',
  cfg: 'InvitationConf',
  obj (input: { address: string; module: string; cfg: string }) {
    return `${input.address}::${input.module}::${input.cfg}`;
  },
};

const market = {
  orderBookAddress: ROOCH_MULTI_SIG_ADDRESS,
  tickInfo: {
    grow: {
      obj: '0x4dc9dde9dc7eabe0eb66913a09e8e47dc952771b9172824062d60670c91e35f6',
    },
    gold: {
      obj: '0xf8a12cc79615988ef0f04d8542b18fe27d5f967972e30fd89328c37f5da9f288',
    },
  } as {
    [key: string]: {
      obj: string;
    }
  }
};

const { networkConfig, useNetworkVariable, useNetworkVariables } = createNetworkConfig({
  mainnet: {
    url: getRoochNodeUrl('mainnet'),
    variables: {
      roochMultiSigAddr: ROOCH_MULTI_SIG_ADDRESS,
      redEnvelope: {
        address: '0x1d6f6657fc996008a1e43b8c13805e969a091560d4cea57b1db9f3ce4450d977',
      },
      gasMarket: {
        address: ROOCH_MULTI_SIG_ADDRESS,
        recipientBTCAddress: 'bc1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4sugkfjt',
        memPool: 'https://mempool.space/tx/',
      },
      faucet: faucet('main'),
      inviter,
      market,
    },
  },
  testnet: {
    url: getRoochNodeUrl('testnet'),
    variables: {
      roochMultiSigAddr: ROOCH_MULTI_SIG_ADDRESS,
      redEnvelope: {
        address: '0x1d6f6657fc996008a1e43b8c13805e969a091560d4cea57b1db9f3ce4450d977',
      },
      gasMarket: {
        address: ROOCH_MULTI_SIG_ADDRESS,
        recipientBTCAddress: 'tb1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4stqqxgy',
        memPool: 'https://mempool.space/testnet/tx/',
      },
      faucet: faucet('test'),
      inviter,
      market,
    },
  },
  localnet: {
    url: getRoochNodeUrl('localnet'),
    variables: {
      roochMultiSigAddr: ROOCH_MULTI_SIG_ADDRESS,
      redEnvelope: {
        address: '0x1d6f6657fc996008a1e43b8c13805e969a091560d4cea57b1db9f3ce4450d977',
      },
      gasMarket: {
        address: ROOCH_MULTI_SIG_ADDRESS,
        recipientBTCAddress: 'tb1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4stqqxgy',
        memPool: 'https://mempool.space/testnet/tx/',
      },
      faucet: faucet('test'),
      inviter,
      market,
    },
  },
});

export { networkConfig, useNetworkVariable, useNetworkVariables };
