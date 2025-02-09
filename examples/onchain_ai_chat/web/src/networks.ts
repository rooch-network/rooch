import { getRoochNodeUrl } from '@roochnetwork/rooch-sdk'
import { createNetworkConfig } from "@roochnetwork/rooch-sdk-kit"

import { DEVNET_PACKAGE_ID, MAINNET_PACKAGE_ID } from './constants'

const { networkConfig, useNetworkVariable, useNetworkVariables } =
  createNetworkConfig({
    mainnet: {
      url: getRoochNodeUrl("mainnet"),
      variables: {
        packageId: MAINNET_PACKAGE_ID,
      },
    },
    devnet: {
      url: getRoochNodeUrl("devnet"),
      variables: {
        packageId: DEVNET_PACKAGE_ID,
      },
    },
    testnet: {
      url: getRoochNodeUrl("testnet"),
      variables: {
        packageId: DEVNET_PACKAGE_ID,
      },
    },
    localnet: {
      url: getRoochNodeUrl("localnet"),
      variables: {
        packageId: DEVNET_PACKAGE_ID,
      },
    },
  })

export { useNetworkVariable, useNetworkVariables, networkConfig }