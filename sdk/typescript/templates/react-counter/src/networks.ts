import { getRoochNodeUrl } from '@roochnetwork/rooch-sdk'
import { createNetworkConfig } from "@roochnetwork/rooch-sdk-kit"

import { DEVNET_COUNTER_PACKAGE_ID, MAINNET_COUNTER_PACKAGE_ID } from './constants.ts'

const { networkConfig, useNetworkVariable, useNetworkVariables } =
  createNetworkConfig({
    devnet: {
      url: getRoochNodeUrl("devnet"),
      variables: {
        counterPackageId: DEVNET_COUNTER_PACKAGE_ID,
      },
    },
    testnet: {
      url: getRoochNodeUrl("testnet"),
      variables: {
        counterPackageId: DEVNET_COUNTER_PACKAGE_ID,
      },
    },
    localnet: {
      url: getRoochNodeUrl("localnet"),
      variables: {
        counterPackageId: MAINNET_COUNTER_PACKAGE_ID,
      },
    },
  })

export { useNetworkVariable, useNetworkVariables, networkConfig }
