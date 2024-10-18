// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { BitcoinAddress, Bytes, ThirdPartyAddress, str, bytes } from '@roochnetwork/rooch-sdk'

import { BitcoinWallet } from '../wellet/index.js'
import { All_NETWORK, WalletNetworkType } from './types.js'

export class UniSatWallet extends BitcoinWallet {
  getName(): string {
    return 'UniSat'
  }

  getIcon(_?: 'dark' | 'light'): string {
    return 'data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiPz4KPHN2ZyBpZD0iX2ZyYW1lXzIiIGRhdGEtbmFtZT0iZnJhbWUgMiIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiB4bWxuczp4bGluaz0iaHR0cDovL3d3dy53My5vcmcvMTk5OS94bGluayIgdmlld0JveD0iLTE1IDAgMTAwIDEwMCI+CiAgPGRlZnM+CiAgICA8c3R5bGU+CiAgICAgIC5jbHMtMSB7CiAgICAgICAgZmlsbDogdXJsKCNfbGxfMTI2KTsKICAgICAgfQoKICAgICAgLmNscy0yIHsKICAgICAgICBmaWxsOiB1cmwoI19sbF8xMjMpOwogICAgICB9CgogICAgICAuY2xzLTMgewogICAgICAgIGZpbGw6IHVybCgjX2xsXzEyMSk7CiAgICAgIH0KCiAgICAgIC5jbHMtNCB7CiAgICAgICAgZmlsbDogI2ZmZjsKICAgICAgICBmb250LWZhbWlseTogSmV0QnJhaW5zTW9ub1JvbWFuLU1lZGl1bSwgJ0pldEJyYWlucyBNb25vJzsKICAgICAgICBmb250LXNpemU6IDI0Ljc5cHg7CiAgICAgICAgZm9udC12YXJpYXRpb24tc2V0dGluZ3M6ICd3Z2h0JyA1MDA7CiAgICAgIH0KICAgIDwvc3R5bGU+CiAgICA8bGluZWFyR3JhZGllbnQgaWQ9Il9sbF8xMjYiICB4MT0iOTYxLjY4IiB5MT0iLTQ1LjU3IiB4Mj0iOTg2LjE0IiB5Mj0iLTExMC4wNiIgZ3JhZGllbnRUcmFuc2Zvcm09InRyYW5zbGF0ZSg3ODAuOTkgNjcxLjcpIHJvdGF0ZSgtMTM0LjczKSIgZ3JhZGllbnRVbml0cz0idXNlclNwYWNlT25Vc2UiPgogICAgICA8c3RvcCBvZmZzZXQ9IjAiIHN0b3AtY29sb3I9IiMwNzAxMDAiLz4KICAgICAgPHN0b3Agb2Zmc2V0PSIuMzYiIHN0b3AtY29sb3I9IiM3NzM5MGQiLz4KICAgICAgPHN0b3Agb2Zmc2V0PSIuNjciIHN0b3AtY29sb3I9IiNlYTgxMDEiLz4KICAgICAgPHN0b3Agb2Zmc2V0PSIxIiBzdG9wLWNvbG9yPSIjZjRiODUyIi8+CiAgICA8L2xpbmVhckdyYWRpZW50PgogICAgPGxpbmVhckdyYWRpZW50IGlkPSJfbGxfMTIxIiAgeDE9Ijk2NS4xNyIgeTE9Ii0xMzIuNDEiIHgyPSI5MjkuMjIiIHkyPSItNjUuMjIiIGdyYWRpZW50VHJhbnNmb3JtPSJ0cmFuc2xhdGUoNzgwLjk5IDY3MS43KSByb3RhdGUoLTEzNC43MykiIGdyYWRpZW50VW5pdHM9InVzZXJTcGFjZU9uVXNlIj4KICAgICAgPHN0b3Agb2Zmc2V0PSIwIiBzdG9wLWNvbG9yPSIjMDcwMTAwIi8+CiAgICAgIDxzdG9wIG9mZnNldD0iLjM3IiBzdG9wLWNvbG9yPSIjNzczOTBkIi8+CiAgICAgIDxzdG9wIG9mZnNldD0iLjY3IiBzdG9wLWNvbG9yPSIjZWE4MTAxIi8+CiAgICAgIDxzdG9wIG9mZnNldD0iMSIgc3RvcC1jb2xvcj0iI2Y0ZmI1MiIvPgogICAgPC9saW5lYXJHcmFkaWVudD4KICAgIDxyYWRpYWxHcmFkaWVudCBpZD0iX2xsXzEyMyIgIGN4PSIzNS41OSIgY3k9IjMwLjc2IiBmeD0iMzUuNTkiIGZ5PSIzMC43NiIgcj0iNy40NyIgZ3JhZGllbnRUcmFuc2Zvcm09InRyYW5zbGF0ZSgwIDApIiBncmFkaWVudFVuaXRzPSJ1c2VyU3BhY2VPblVzZSI+CiAgICAgIDxzdG9wIG9mZnNldD0iMCIgc3RvcC1jb2xvcj0iI2Y0Yjg1MiIvPgogICAgICA8c3RvcCBvZmZzZXQ9Ii4zMyIgc3RvcC1jb2xvcj0iI2VhODEwMSIvPgogICAgICA8c3RvcCBvZmZzZXQ9Ii42NCIgc3RvcC1jb2xvcj0iIzc3MzkwZCIvPgogICAgICA8c3RvcCBvZmZzZXQ9IjEiIHN0b3AtY29sb3I9IiMwNzAxMDAiLz4KICAgIDwvcmFkaWFsR3JhZGllbnQ+CiAgPC9kZWZzPgogIDxnIGlkPSJfZnJhbWVfMS0yIiA+CiAgICA8Zz4KICAgICAgPGc+CiAgICAgICAgPHBhdGggY2xhc3M9ImNscy0xIiBkPSJNNTQuODEsOC45MWwyMC4zNCwyMC4xNGMxLjczLDEuNzEsMi41OCwzLjQ0LDIuNTUsNS4xOS0uMDMsMS43NC0uNzcsMy4zNC0yLjIzLDQuNzgtMS41MiwxLjUxLTMuMTYsMi4yOC00LjkyLDIuMzEtMS43NiwuMDMtMy41LS44Mi01LjI0LTIuNTNsLTIwLjgtMjAuNmMtMi4zNi0yLjM0LTQuNjQtNC02Ljg0LTQuOTctMi4xOS0uOTctNC41LTEuMTItNi45Mi0uNDYtMi40MiwuNjYtNS4wMiwyLjM3LTcuOCw1LjEzLTMuODQsMy44LTUuNjcsNy4zNy01LjQ4LDEwLjcxLC4xOSwzLjM0LDIuMDksNi43OSw1LjcxLDEwLjM4bDIwLjk3LDIwLjc3YzEuNzUsMS43MywyLjYxLDMuNDYsMi41OCw1LjE4LS4wMywxLjcyLS43OCwzLjMyLTIuMjYsNC43OC0xLjQ4LDEuNDYtMy4xLDIuMjMtNC44OCwyLjI5LTEuNzcsLjA2LTMuNTMtLjc4LTUuMjgtMi41MUwxMy45OSw0OS4zNmMtMy4zMS0zLjI4LTUuNy02LjM4LTcuMTctOS4zLTEuNDctMi45Mi0yLjAyLTYuMjMtMS42NC05LjkyLC4zNC0zLjE2LDEuMzYtNi4yMiwzLjA0LTkuMTksMS42OS0yLjk3LDQuMS02LDcuMjMtOS4xMSwzLjczLTMuNyw3LjI5LTYuNTMsMTAuNjktOC41QzI5LjU0LDEuMzcsMzIuODIsLjI3LDM1Ljk5LC4wNGMzLjE3LS4yMyw2LjMsLjQsOS40LDEuODksMy4wOSwxLjQ5LDYuMjMsMy44MSw5LjQzLDYuOThaIi8+CiAgICAgICAgPHBhdGggY2xhc3M9ImNscy0zIiBkPSJNMjIuOTIsOTAuMTlMMi41OCw3MC4wNUMuODUsNjguMzQsMCw2Ni42MSwuMDMsNjQuODZzLjc3LTMuMzQsMi4yMy00Ljc4YzEuNTItMS41MSwzLjE2LTIuMjgsNC45Mi0yLjMxLDEuNzYtLjAzLDMuNSwuODEsNS4yNCwyLjUzbDIwLjgsMjAuNmMyLjM3LDIuMzQsNC42NCw0LDYuODQsNC45N3M0LjUsMS4xMiw2LjkyLC40NmMyLjQyLS42Niw1LjAyLTIuMzcsNy44LTUuMTMsMy44NC0zLjgsNS42Ny03LjM3LDUuNDgtMTAuNzEtLjE5LTMuMzQtMi4wOS02LjgtNS43MS0xMC4zOGwtMTEuMTctMTAuOTdjLTEuNzUtMS43My0yLjYxLTMuNDYtMi41OC01LjE4LC4wMy0xLjcyLC43OC0zLjMyLDIuMjYtNC43OCwxLjQ4LTEuNDYsMy4xLTIuMjMsNC44OC0yLjI5LDEuNzctLjA2LDMuNTMsLjc4LDUuMjgsMi41MWwxMC41MywxMC4zNGMzLjMxLDMuMjgsNS43LDYuMzgsNy4xNyw5LjMsMS40NywyLjkyLDIuMDIsNi4yMywxLjY0LDkuOTItLjM0LDMuMTYtMS4zNiw2LjIyLTMuMDQsOS4xOS0xLjY5LDIuOTctNC4xLDYtNy4yMyw5LjExLTMuNzMsMy43LTcuMjksNi41My0xMC42OSw4LjUtMy40LDEuOTctNi42OCwzLjA3LTkuODUsMy4zLTMuMTcsLjIzLTYuMy0uNC05LjQtMS44OS0zLjA5LTEuNDktNi4yNC0zLjgxLTkuNDMtNi45OFoiLz4KICAgICAgICA8Y2lyY2xlIGNsYXNzPSJjbHMtMiIgY3g9IjM1LjYiIGN5PSIzMC43NSIgcj0iNy40NyIvPgogICAgICA8L2c+CiAgICA8L2c+CiAgPC9nPgo8L3N2Zz4K'
  }

  getDescription(): string {
    return 'UniSat Wallet'
  }

  getInstallUrl(): string {
    return ''
  }

  getTarget(): any {
    return (window as any).unisat
  }

  async connect(): Promise<ThirdPartyAddress[]> {
    let addresses: string[] = await this.getTarget().getAccounts()

    if (!addresses || addresses.length === 0) {
      await this.getTarget().requestAccounts()
      return this.connect()
    }

    let publicKey = await this.getTarget().getPublicKey()

    this.address = addresses.map((item) => new BitcoinAddress(item))
    this.currentAddress = this.address[0]
    this.publicKey = publicKey

    return this.address
  }

  switchNetwork(network: WalletNetworkType): void {
    this.getTarget().switchNetwork(network)
  }
  getNetwork(): WalletNetworkType {
    return this.getTarget().getNetwork()
  }

  getSupportNetworks(): WalletNetworkType[] {
    return All_NETWORK
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

  async sign(msg: Bytes): Promise<Bytes> {
    const msgStr = str('utf8', msg)
    const sign = await this.getTarget().signMessage(msgStr)
    return bytes('base64', sign).subarray(1) // remove recover id
  }

  sendBtc(input: {
    toAddress: string
    satoshis: number
    options?: { feeRate: number }
  }): Promise<string> {
    return this.getTarget().sendBitcoin(input.toAddress, input.satoshis, input.options)
  }

  getBalance(): Promise<{ confirmed: number; unconfirmed: number; total: string }> {
    return this.getTarget().getBalance()
  }
}
