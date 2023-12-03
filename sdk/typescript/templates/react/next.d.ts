// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import MetaMaskSDK from '@metamask/sdk'

declare global {
  interface Window {
    sdkProvider: MetaMaskSDK
    ethereum?: MetaMaskSDK
  }
}
