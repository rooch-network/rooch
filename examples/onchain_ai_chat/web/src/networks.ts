import { getRoochNodeUrl } from '@roochnetwork/rooch-sdk'
import { createNetworkConfig } from "@roochnetwork/rooch-sdk-kit"

import { PACKAGE_ID } from './constants'

const { networkConfig, useNetworkVariable, useNetworkVariables } =
  createNetworkConfig({
    mainnet: {
      url: getRoochNodeUrl("mainnet"),
      variables: {
        packageId: PACKAGE_ID,
      },
    },
    devnet: {
      url: getRoochNodeUrl("devnet"),
      variables: {
        packageId: PACKAGE_ID,
      },
    },
    testnet: {
      url: getRoochNodeUrl("testnet"),
      variables: {
        packageId: PACKAGE_ID,
      },
    },
    localnet: {
      url: getRoochNodeUrl("localnet"),
      variables: {
        packageId: PACKAGE_ID,
      },
    },
  })

export { useNetworkVariable, useNetworkVariables, networkConfig }