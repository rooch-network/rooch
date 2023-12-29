// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { base58_to_binary } from 'base58-js'
import { bech32, bech32m } from 'bech32'
import { sha3_256 } from '@noble/hashes/sha3'

enum Network {
  mainnet = 'mainnet',
  testnet = 'testnet',
  regtest = 'regtest',
}

enum AddressType {
  p2pkh = 'p2pkh',
  p2sh = 'p2sh',
  p2wpkh = 'p2wpkh',
  p2wsh = 'p2wsh',
  p2tr = 'p2tr',
}

type AddressInfo = {
  bytes: Uint8Array
  bech32: boolean
  network: Network
  address: string
  type: AddressType
}

const addressTypes: { [key: number]: { type: AddressType; network: Network } } = {
  0x00: {
    type: AddressType.p2pkh,
    network: Network.mainnet,
  },

  0x6f: {
    type: AddressType.p2pkh,
    network: Network.testnet,
  },

  0x05: {
    type: AddressType.p2sh,
    network: Network.mainnet,
  },

  0xc4: {
    type: AddressType.p2sh,
    network: Network.testnet,
  },
}

const parseBech32 = (address: string): AddressInfo => {
  let decoded

  try {
    if (address.startsWith('bc1p') || address.startsWith('tb1p') || address.startsWith('bcrt1p')) {
      decoded = bech32m.decode(address)
    } else {
      decoded = bech32.decode(address)
    }
  } catch (error) {
    throw new Error('Invalid address')
  }

  const mapPrefixToNetwork: { [key: string]: Network } = {
    bc: Network.mainnet,
    tb: Network.testnet,
    bcrt: Network.regtest,
  }

  const network: Network = mapPrefixToNetwork[decoded.prefix]

  if (network === undefined) {
    throw new Error('Invalid address')
  }

  const witnessVersion = decoded.words[0]

  if (witnessVersion < 0 || witnessVersion > 16) {
    throw new Error('Invalid address')
  }
  const data = bech32.fromWords(decoded.words.slice(1))

  let type

  if (data.length === 20) {
    type = AddressType.p2wpkh
  } else if (witnessVersion === 1) {
    type = AddressType.p2tr
  } else {
    type = AddressType.p2wsh
  }

  let bytes = new Uint8Array(21)
  bytes.set([0])
  bytes.set(data, 1)

  return {
    bytes: bytes,
    bech32: true,
    network,
    address,
    type,
  }
}

const getAddressInfo = (address: string): AddressInfo => {
  let decoded: Uint8Array
  const prefix = address.substr(0, 2).toLowerCase()

  if (prefix === 'bc' || prefix === 'tb') {
    return parseBech32(address)
  }

  try {
    decoded = base58_to_binary(address)
  } catch (error) {
    throw new Error('Invalid address')
  }

  const { length } = decoded

  if (length !== 25) {
    throw new Error('Invalid address')
  }

  const version = decoded[0]

  const checksum = decoded.slice(length - 4, length)
  const body = decoded.slice(0, length - 4)

  const expectedChecksum = sha3_256(sha3_256(body)).slice(0, 4)

  if (checksum.some((value: number, index: number) => value !== expectedChecksum[index])) {
    throw new Error('Invalid address')
  }

  const validVersions = Object.keys(addressTypes).map(Number)

  if (!validVersions.includes(version)) {
    throw new Error('Invalid address')
  }

  const addressType = addressTypes[version]

  return {
    bytes: decoded,
    ...addressType,
    address,
    bech32: false,
  }
}

const validate = (address: string, network?: Network) => {
  try {
    const addressInfo = getAddressInfo(address)

    if (network) {
      return network === addressInfo.network
    }

    return true
  } catch (error) {
    return false
  }
}

export { getAddressInfo, Network, AddressType, validate }

export type { AddressInfo }

export default validate
