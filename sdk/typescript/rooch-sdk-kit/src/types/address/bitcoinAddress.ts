// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { base58_to_binary } from 'base58-js'
import { bech32, bech32m } from 'bech32'
import { createHash } from 'sha256-uint8array'

const sha256 = (payload: Uint8Array) => createHash().update(payload).digest()

enum Network {
  mainnet = 'mainnet',
  testnet = 'testnet',
  regtest = 'regtest',
}

// The method used to distinguish bitcoin address payload type.
// Ref: BitcoinAddressPayloadType https://github.com/rooch-network/rooch/blob/main/crates/rooch-types/src/address.rs
enum AddressType {
  p2pkh = 0,
  p2sh = 1,
  p2wpkh = 2,
  p2wsh = 2,
  p2tr = 2,
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

  // replace version & add witness version
  let bytes = new Uint8Array(data.length + 2)
  bytes.set([type])
  bytes.set([witnessVersion], 1)
  bytes.set(data, 2)

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

  const expectedChecksum = sha256(sha256(body)).slice(0, 4)

  if (checksum.some((value: number, index: number) => value !== expectedChecksum[index])) {
    throw new Error('Invalid address')
  }

  const validVersions = Object.keys(addressTypes).map(Number)

  if (!validVersions.includes(version)) {
    throw new Error('Invalid address')
  }

  const addressType = addressTypes[version]

  // replace version
  let bytes = new Uint8Array(body.length)
  bytes.set([addressType.type])
  bytes.set(body.slice(1), 1)

  return {
    bytes: bytes,
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
