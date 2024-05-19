// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { UseMutationOptions, UseMutationResult } from '@tanstack/react-query'
import { useMutation } from '@tanstack/react-query'
import { IAccount } from '@roochnetwork/rooch-sdk'
import { roochMutationKeys } from '../../constants/roochMutationKeys'
import { useRoochClient } from './index'

type UseTransferObjectArgs = {
  account: IAccount
  toAddress: string
  objId: string
  objType: string
}

type UseTransferObjectResult = void

type UseSwitchNetworkMutationOptions = Omit<
  UseMutationOptions<UseTransferObjectResult, Error, UseTransferObjectArgs, unknown>,
  'mutationFn'
>

export function useTransferObject({
  mutationKey,
  ...mutationOptions
}: UseSwitchNetworkMutationOptions = {}): UseMutationResult<
  UseTransferObjectResult,
  Error,
  UseTransferObjectArgs,
  unknown
> {
  const client = useRoochClient()

  return useMutation({
    mutationKey: roochMutationKeys.transferObject(mutationKey),
    mutationFn: async (args) => {
      const struct = args.objType.split('::')

      if (struct.length !== 3) {
        console.log('type args is error')
        return
      }

      const result = await client.executeTransaction({
        address: args.account.getAddress(),
        authorizer: args.account.getAuthorizer(),
        funcId: '0x3::transfer::transfer_object',
        args: [
          {
            type: 'Address',
            value: args.toAddress,
          },
          {
            type: 'ObjectID',
            value: args.objId,
          },
        ],
        tyArgs: [
          {
            Struct: {
              address: struct[0],
              module: struct[1],
              name: struct[2],
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
