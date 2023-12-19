// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export class WalletAccount {
  private readonly address: string

  public constructor(address: string) {
    this.address = address
  }

  /**
   * Get account address
   */
  public getAddress(): string {
    return this.address
  }
}
