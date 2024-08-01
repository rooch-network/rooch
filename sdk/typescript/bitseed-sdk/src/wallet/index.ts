import { Network } from '../types'
import { Ordit, AddressFormats } from '@sadoprotocol/ordit-sdk'

export type WalletOptions = {
  wif?: string;
  seed?: string;
  privateKey?: string;
  bip39?: string;
  network?: Network;
  type?: AddressFormats;
};

export class Wallet extends Ordit {
  constructor(opts: WalletOptions) {
    super(opts)
  }
}