// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, it, afterAll } from 'vitest'
import { TestBox } from '../setup.js'
import { Args, bcs } from '../../src/bcs/index.js'
import { BitcoinAddress, BitcoinNetowkType, fromHEX, getRoochNodeUrl, RoochAddress, StateKVView } from "../../src";

describe('Checkpoints Transfer API', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = TestBox.setup(getRoochNodeUrl('mainnet'))
  })

  afterAll(async () => {
    testBox.cleanEnv()
  })

  type VoterInfo = {
    voteTime: number
    address: string
    value: number
  }

  it('export all votes', async () => {
    let votes: VoterInfo[] = []

    // find 2025-1-5-20 the closest deal
    // const txResult = await testBox.getClient().queryTransactions({
    //   filter: {
    //     time_range: {
    //       start_time: (1736078340*1000).toString(),
    //       end_time: (1736078400 * 1000).toString()
    //     }
    //   }
    // })
    //
    // txResult.data.sort((a, b) => Number(a.transaction.sequence_info.tx_timestamp) - Number(b.transaction.sequence_info.tx_timestamp))
    // tx order 72730361
    // tx hash 0x54adf5ddc03b1083f9146645e1d7ecaa5b272a66c449a96ec82f38073713c382
    // state root 0x42d23fdaaf5ec6cd62c7f6f2ba527e397ea3d45d8c1f9d5956054aa8b8122271
    const getAllVoters = async (table: string, cursor?: string) => {
      const result = await testBox.getClient().listStates({
        accessPath: `/table/${table}`,
        stateOption: {
          stateRoot: '0x42d23fdaaf5ec6cd62c7f6f2ba527e397ea3d45d8c1f9d5956054aa8b8122271',
        },
        cursor,
        limit: '200',
      })

      const items = result.data.map((item) => {
        //0x292467201e8b21f9188f722df170854e97681353da66ec16586817abaad96c36a301000000000000000000000000000000000000000000000000000000000000
        const d = bcs.struct('xxx', {
          address: bcs.bytes(32),
          value: bcs.u256(),
        })

        const view = item.state.value
        const decode = d.parse(fromHEX(view))
        return {
          voteTime: Number(item.state.created_at),
          address: new RoochAddress(decode.address).toHexAddress(),
          value: Number(decode.value),
        }
      })
      votes = votes.concat(items).sort((a, b) => b.value - a.value)
      if (result.has_next_page) {
        await getAllVoters(table, result.next_cursor || undefined)
      } else {
      }
    }

    await getAllVoters('0x8b9d2a598a1f2d9cbdec84a479360afb8431ef5b19f871bf36d9de8ad19d9041')

    const endTime = 1736078400 * 1000
    votes = votes.filter((item) => item.voteTime < endTime)

    const with1000 = votes.slice(0, 1000)

    const resultAddressMap = await testBox.getClient().executeViewFunction({
      target: '0x3::address_mapping::resolve_bitcoin_batch',
      args: [
        Args.vec(
          'address',
          with1000.map((item) => item.address),
        ),
      ],
    })

    const warpWith1000 = with1000.map((item, i) => {
      const tmp = (resultAddressMap.return_values![0].decoded_value as any).value[i][0]
      const btcAddress = new BitcoinAddress(tmp, BitcoinNetowkType.Bitcoin).toStr()
      return {
        ...item,
        btcAddress,
        roochAddress: new RoochAddress(item.address).toBech32Address(),
      }
    })

    const json = JSON.stringify(warpWith1000)

    console.log(json)
  })

  it('export all register', async () => {
    // const result = await testBox.getClient().queryObjectStates({
    //  filter: {
    //    object_type: '0x701c21bf1c8cd5af8c42983890d8ca55e7a820171b8e744c13f2d9998bf76cc3::grow_registration::Registration'
    //  }
    // })

    //main 0xa1e5d1779e9764839a0526e35d5972d28584d3dcfac1d2305e231b6490a59740
    //test 0xa21c5dd091454a554bacde8210bfe0aed7d392af04b04d26a155c8f8cfeac9dc
    let votes: StateKVView[] = []
    let nextCursor: string | undefined | null = undefined
    while (true) {
      const result = await testBox.getClient().listStates({
        accessPath: '/table/0xa1e5d1779e9764839a0526e35d5972d28584d3dcfac1d2305e231b6490a59740',
        limit: '200',
        cursor: nextCursor
      })
      nextCursor = result.next_cursor
      votes = votes.concat(result.data)

      if (!result.has_next_page) {
        break
      }
    }

    const formatVotes = votes.map((item) => {
      const view = item.state.decoded_value!.value! as any
      return {
        address: view.name,
        roochAddress: new RoochAddress(view.name).toBech32Address(),
        voteCount: view.value!.value!.amount,
        recipientAddress: view.value!.value!.register_info,
      }
    })

    const resultAddressMap = await testBox.getClient().executeViewFunction({
      target: '0x3::address_mapping::resolve_bitcoin_batch',
      args: [
        Args.vec(
          'address',
          formatVotes.map((item) => item.address),
        ),
      ],
    })

    const finalVotes = formatVotes.map((item, i) => {
      const tmp = (resultAddressMap.return_values![0].decoded_value as any).value[i][0]
      const btcAddress = new BitcoinAddress(tmp, BitcoinNetowkType.Bitcoin).toStr()
      return {
        ...item,
        btcAddress: btcAddress,
      }
    })

    const json = JSON.stringify(finalVotes)

    console.log(json)
  })
})
