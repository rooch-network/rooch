import { getRoochNodeUrl } from '@roochnetwork/rooch-sdk'
import { createNetworkConfig } from '@roochnetwork/rooch-sdk-kit'

const { networkConfig, useNetworkVariable, useNetworkVariables } = createNetworkConfig({
  mainnet: {
    url: getRoochNodeUrl('mainnet'),
    variables: {
      counterPackageId: '',
    },
  },
  devnet: {
    url: getRoochNodeUrl('devnet'),
    variables: {
      counterPackageId: '',
    },
  },
  testnet: {
    url: getRoochNodeUrl('testnet'),
    variables: {
      counterPackageId: '',
    },
  },
  localnet: {
    url: getRoochNodeUrl('localnet'),
    variables: {
      counterPackageId: '',
    },
  },
})

export { useNetworkVariable, useNetworkVariables, networkConfig }
