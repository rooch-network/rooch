// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { BitcoinAddress, Bytes, ThirdPartyAddress, str, bytes } from '@roochnetwork/rooch-sdk'
import { BitcoinWallet } from '../wellet/index.js'

export class OkxWallet extends BitcoinWallet {
  getName(): string {
    return 'OKX'
  }

  getIcon(_?: 'dark' | 'light'): string {
    // if (theme === 'dark') {
    //   return ''
    // }

    return 'data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiPz48c3ZnIGlkPSJhIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxNTAgMTUwIj48ZGVmcz48c3R5bGU+LmV7ZmlsbDpub25lO308L3N0eWxlPjwvZGVmcz48ZyBpZD0iYiI+PGcgaWQ9ImMiPjxwYXRoIGlkPSJkIiBjbGFzcz0iZSIgZD0iTTAsMEgxNTBWMTUwSDBWMFoiLz48L2c+PC9nPjxwYXRoIGQ9Ik0xMy44MSwxMy41N3YxMjMuOThoMTIzLjk4VjEzLjU3SDEzLjgxWm0yNi44MiwyOC42NGMwLS44NywuNzEtMS41OCwxLjU4LTEuNThoMjAuM2MuODcsMCwxLjU4LC43MSwxLjU4LDEuNTh2MjAuM2MwLC44OC0uNzEsMS41OS0xLjU4LDEuNTloLTIwLjNjLS44NywwLTEuNTgtLjcxLTEuNTgtMS41OXYtMjAuM1ptMjMuNDYsNjYuN2MwLC44Ny0uNzEsMS41OC0xLjU4LDEuNThoLTIwLjNjLS44NywwLTEuNTgtLjcxLTEuNTgtMS41OHYtMjAuM2MwLS44OCwuNzEtMS41OSwxLjU4LTEuNTloMjAuM2MuODcsMCwxLjU4LC43MSwxLjU4LDEuNTl2MjAuM1ptMjEuODYtMjEuNjJoLTIwLjNjLS44NywwLTEuNTktLjcxLTEuNTktMS41OXYtMjAuM2MwLS44NywuNzEtMS41OSwxLjU5LTEuNTloMjAuM2MuODcsMCwxLjU5LC43MSwxLjU5LDEuNTl2MjAuM2MwLC44Ny0uNzEsMS41OS0xLjU5LDEuNTlabTI1LjA1LDIxLjYyYzAsLjg3LS43MSwxLjU4LTEuNTksMS41OGgtMjAuM2MtLjg3LDAtMS41OC0uNzEtMS41OC0xLjU4di0yMC4zYzAtLjg4LC43MS0xLjU5LDEuNTgtMS41OWgyMC4zYy44NywwLDEuNTksLjcxLDEuNTksMS41OXYyMC4zWm0wLTQ2LjQxYzAsLjg4LS43MSwxLjU5LTEuNTksMS41OWgtMjAuM2MtLjg3LDAtMS41OC0uNzEtMS41OC0xLjU5di0yMC4zYzAtLjg3LC43MS0xLjU4LDEuNTgtMS41OGgyMC4zYy44NywwLDEuNTksLjcxLDEuNTksMS41OHYyMC4zWiIvPjwvc3ZnPg=='
  }

  getDescription(): string {
    return 'OKX Wallet'
  }

  getInstallUrl(): string {
    return 'https://chromewebstore.google.com/detail/okx-wallet/mcohilncbfahbmgdjkbpemcciiolgcge'
  }

  async sign(msg: Bytes): Promise<Bytes> {
    const msgStr = str('utf8', msg)
    const sign = await this.getTarget().signMessage(msgStr, {
      from: this.currentAddress?.toStr(),
    })
    return bytes('base64', sign).subarray(1)
  }

  getTarget(): any {
    return (window as any).okxwallet?.bitcoin
  }

  async connect(): Promise<ThirdPartyAddress[]> {
    const obj = await this.getTarget().connect()
    this.currentAddress = new BitcoinAddress(obj.address)
    this.publicKey = obj.compressedPublicKey !== '' ? obj.compressedPublicKey : obj.publicKey
    this.address = [this.currentAddress]

    return this.address
  }

  switchNetwork(): void {
    this.getTarget().switchNetwork()
  }

  getNetwork(): string {
    return this.getTarget().getNetwork()
  }

  getSupportNetworks(): string[] {
    return ['livenet']
  }

  onAccountsChanged(callback: (account: string[]) => void): void {
    this.getTarget().on('accountsChanged', callback)
  }

  removeAccountsChanged(callback: (account: string[]) => void): void {
    this.getTarget().removeListener('accountsChanged', callback)
  }

  onNetworkChanged(callback: (network: string) => void): void {
    this.getTarget().on('networkChanged', callback)
  }

  removeNetworkChanged(callback: (network: string) => void): void {
    this.getTarget().removeListener('networkChanged', callback)
  }
}
