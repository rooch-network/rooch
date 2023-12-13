// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import WebView from 'react-native-webview'
import { SDKProvider } from '@metamask/sdk/dist/browser/es/src/provider/SDKProvider'
import { MetaMaskSDK } from '@metamask/sdk/dist/browser/es/src/sdk'
import { MetaMaskInpageProvider } from '@metamask/providers'

declare global {
  interface Window {
    ReactNativeWebView?: WebView
    sdkProvider: SDKProvider
    ethereum?: SDKProvider
    mmsdk?: MetaMaskSDK
    extension?: MetaMaskInpageProvider
    extensions?: any[]
    MSStream: unknown
  }
}
