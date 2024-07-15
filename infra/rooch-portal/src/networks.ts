// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { getRoochNodeUrl } from '@roochnetwork/rooch-sdk'
import { createNetworkConfig } from '@roochnetwork/rooch-sdk-kit'

import { ROOCH_MINT_OPERATING_ADDRESS, ROOCH_NFT_OPERATING_ADDRESS } from '@/common/constant.ts'

const { networkConfig, useNetworkVariable, useNetworkVariables } =
  createNetworkConfig({
    testnet: {
      url: getRoochNodeUrl("testnet"),
      variables: {
        roochOperatingAddress: ROOCH_NFT_OPERATING_ADDRESS,
        mintAddress: ROOCH_MINT_OPERATING_ADDRESS
      },
    },
    localnet: {
      url: getRoochNodeUrl("localnet"),
      variables: {
        roochOperatingAddress: ROOCH_NFT_OPERATING_ADDRESS,
        mintAddress: ROOCH_MINT_OPERATING_ADDRESS,
      },
    }
  });

export { useNetworkVariable, useNetworkVariables, networkConfig };
