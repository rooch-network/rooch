// // Copyright (c) RoochNetwork
// // SPDX-License-Identifier: Apache-2.0
//
// import onekey from '@onekeyfe/hd-web-sdk'
//
// import { BitcoinAddress, Bytes, ThirdPartyAddress, str, bytes } from '@roochnetwork/rooch-sdk'
// import { BitcoinWallet } from '../wellet/index.js'
//
// type Device = {
//   connectId: string | null
//   uuid: string
//   deviceId: string | null
//   deviceType: string
//   name: string
// }
//
// export class OnekeyHardwareWallet extends BitcoinWallet {
//   devices?: Device[]
//
//   getName(): string {
//     return 'onekey-hardware'
//   }
//
//   async sign(msg: Bytes): Promise<Bytes> {
//     const msgStr = str('utf8', msg)
//     const sign = await this.getTarget().signMessage(msgStr)
//     return bytes('base64', sign).subarray(1)
//   }
//
//   getTarget(): any {
//     return undefined
//   }
//
//   async connect(): Promise<ThirdPartyAddress[]> {
//     if (this.devices && this.devices.length > 0 && !this.currentAddress) {
//       await onekey.HardwareWebSdk.init({
//         debug: true,
//       })
//     }
//
//     const { connectId, deviceId } = this.devices![0]
//
//     const address = await onekey.HardwareWebSdk.btcGetAddress(connectId!, deviceId!, {
//       path: `m/86'/0'/x'/x/x`, // taproot
//       coin: 'btc',
//     })
//     if (!address.success) {
//       throw Error(address.payload.error)
//     }
//
//     const publicKey = await onekey.HardwareWebSdk.btcGetPublicKey(connectId!, deviceId!, {
//       path: `m/86'/0'/x'/x/x`,
//       coin: 'btc',
//     })
//
//     if (!publicKey.success) {
//       throw Error(publicKey.payload.error)
//     }
//
//     this.address = [new BitcoinAddress(address.payload.address)]
//     this.currentAddress = this.address[0]
//     this.publicKey = publicKey.payload.xpub
//
//     return this.address
//   }
//
//   switchNetwork(): void {
//     this.getTarget().switchNetwork()
//   }
//
//   getNetwork(): string {
//     return this.getTarget().getNetwork()
//   }
//
//   getSupportNetworks(): string[] {
//     return ['livenet']
//   }
//
//   onAccountsChanged(callback: (account: string[]) => void): void {
//     this.getTarget().on('accountsChanged', callback)
//   }
//
//   removeAccountsChanged(callback: (account: string[]) => void): void {
//     this.getTarget().removeListener('accountsChanged', callback)
//   }
//
//   onNetworkChanged(callback: (network: string) => void): void {
//     this.getTarget().on('networkChanged', callback)
//   }
//
//   removeNetworkChanged(callback: (network: string) => void): void {
//     this.getTarget().removeListener('networkChanged', callback)
//   }
//
//   async checkInstalled(): Promise<boolean> {
//     // TODO: check feature, Ignore wallets that do not support bitcoin
//     const devices = await onekey.HardwareWebSdk.searchDevices()
//     this.devices = devices.success ? (devices.payload as []) : []
//     return devices.success
//   }
// }
