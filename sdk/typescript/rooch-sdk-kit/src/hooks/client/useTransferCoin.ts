// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { IAccount } from '@roochnetwork/rooch-sdk'
import { roochMutationKeys } from '../../constants/roochMutationKeys'
import { useRoochClient } from './index'

type UseTransferCoinArgs = {
  account: IAccount
  recipient: string
  amount: number
  coinType: string
}

type UseTransferCoinResult = void

type UseSwitchNetworkMutationOptions = Omit<
  UseMutationOptions<UseTransferCoinResult, Error, UseTransferCoinArgs, unknown>,
  'mutationFn'
>

export function useTransferCoin({
  mutationKey,
  ...mutationOptions
}: UseSwitchNetworkMutationOptions = {}): UseMutationResult<
  UseTransferCoinResult,
  Error,
  UseTransferCoinArgs,
  unknown
> {
  const client = useRoochClient()

  return useMutation({
    mutationKey: roochMutationKeys.transferCoin(mutationKey),
    mutationFn: async (args) => {
      const struct = args.coinType.split('::')

      if (struct.length !== 3) {
        console.log('type args is error')
        return
      }

      const result = await client.executeTransaction({
        address: args.account.getAddress(),
        authorizer: args.account.getAuthorizer(),
        funcId: '0x3::transfer::transfer_coin',
        args: [
          {
            type: 'Address',
            value: args.recipient,
          },
          {
            type: 'U256',
            value: BigInt(args.amount),
          },
        ],
        tyArgs: [
          {
            Struct: {
              address: '0x3',
              module: 'gas_coin',
              name: 'GasCoin',
            },
          },
        ],
      })

      if (result.execution_info.status.type !== 'executed') {
        Error('transfer failed' + result.execution_info.status.type)
      }
    },
    ...mutationOptions,
  })
}
