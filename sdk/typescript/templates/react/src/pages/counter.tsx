// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/** React */
import { useEffect, useMemo, useState } from 'react'

/** SDK */
import {
  Ed25519Keypair,
  PrivateKeyAuth,
  IAccount,
  Account,
  encodeMoveCallDataWithETH,
} from '@roochnetwork/rooch-sdk'

/** Hooks */
import { useRooch } from '@/hooks/useRooch'
import { useETH } from '@/hooks/useETH'

const ROOCH_ADDRESS = '0xd46e8dd67c5d32be8058bb8eb970870f07244568'
const devCounterAddress = '0x49ee3cf17a017b331ab2b8a4d40ecc9706f328562f9db63cba625a9c106cdf35'
const devCounterModule = `${devCounterAddress}::counter`
const func = `${devCounterModule}::increase`

export default function Counter() {
  const rooch = useRooch()
  const eth = useETH()

  const [value, setValue] = useState<number>(0)
  const [fetch, setFetch] = useState(true)
  const [loading, setLoading] = useState(false)

  const defaultRoochAccount = useMemo<IAccount>(() => {
    const pk = Ed25519Keypair.generate()

    const authorizer = new PrivateKeyAuth(pk)

    return new Account(rooch.provider!, pk.toRoochAddress(), authorizer)
  }, [rooch.provider])

  useEffect(() => {
    eth.connect()
  })

  useEffect(() => {
    if (loading) {
      return
    }
    const fetchCounterValue = async () => {
      const result = await rooch.provider?.executeViewFunction(`${devCounterModule}::value`)

      if (result?.return_values) {
        setValue(parseInt(String(result.return_values[0].decoded_value)))
      }
    }

    fetchCounterValue().finally(() => setFetch(false))
  }, [rooch, loading])

  const handlerIncreaseWithRooch = () => {
    if (loading) {
      return
    }

    setLoading(true)

    defaultRoochAccount
      .runFunction(func, [], [], { maxGasAmount: 10000 })
      .finally(() => setLoading(false))
  }

  const handlerIncreaseWithEth = async () => {
    if (loading || !eth.activeAccount) {
      return
    }

    setLoading(true)

    const moveCallData = encodeMoveCallDataWithETH(func, [], [])

    const params = [
      {
        from: eth.activeAccount!.address,
        to: ROOCH_ADDRESS,
        gas: '0x76c0', // 30400
        gasPrice: '0x9184e72a000', // 10000000000000
        value: '0x4e72a', // 2441406250
        data: moveCallData,
      },
    ]

    try {
      await ethereum.request({
        method: 'eth_sendTransaction',
        params,
      })
    } catch (e: any) {
      console.log(e)
    } finally {
      setLoading(false)
    }
  }

  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <div className="z-10 max-w-5xl w-full items-center justify-between font-mono text-sm lg:flex">
        <p className="fixed left-0 top-0 flex w-full justify-center border-b border-gray-300 bg-gradient-to-b from-zinc-200 pb-6 pt-8 backdrop-blur-2xl dark:border-neutral-800 dark:bg-zinc-800/30 dark:from-inherit lg:static lg:w-auto  lg:rounded-xl lg:border lg:bg-gray-200 lg:p-4 lg:dark:bg-zinc-800/30">
          Get started by editing&nbsp;
          <code className="font-mono font-bold">src/app/pages/counter.tsx</code>
        </p>
      </div>

      {fetch ? (
        <p>loading</p>
      ) : (
        <>
          <div className="relative flex place-items-center before:absolute before:h-[300px] before:w-[480px] before:-translate-x-1/2 before:rounded-full before:bg-gradient-radial before:from-white before:to-transparent before:blur-2xl before:content-[''] after:absolute after:-z-20 after:h-[180px] after:w-[240px] after:translate-x-1/3 after:bg-gradient-conic after:from-sky-200 after:via-blue-200 after:blur-2xl after:content-[''] before:dark:bg-gradient-to-br before:dark:from-transparent before:dark:to-blue-700 before:dark:opacity-10 after:dark:from-sky-900 after:dark:via-[#0141ff] after:dark:opacity-40 before:lg:h-[360px] z-[-1]">
            <p className="text-black text-4xl font-bold">{value}</p>
          </div>

          <div className="mb-32 grid text-center lg:max-w-5xl lg:w-full lg:mb-0 lg:grid-cols-2 lg:text-left">
            <a
              href="/"
              className="group rounded-lg border border-transparent px-5 py-4 transition-colors hover:border-gray-300 hover:bg-gray-100 hover:dark:border-neutral-700 hover:dark:bg-neutral-800/30 flex flex-col items-center justify-center"
              target="_blank"
              rel="noopener noreferrer"
              onClick={(e) => {
                e.preventDefault()
                handlerIncreaseWithRooch()
              }}
            >
              <h2 className={`mb-3 text-2xl font-semibold`}>
                Increase
                <span className="inline-block transition-transform group-hover:translate-x-1 motion-reduce:transform-none">
                  -&gt;
                </span>
              </h2>
              <p className={`m-0 max-w-[30ch] text-sm opacity-50`}>With rooch sdk</p>
            </a>

            <a
              href="/"
              className="group rounded-lg border border-transparent px-5 py-4 transition-colors hover:border-gray-300 hover:bg-gray-100 hover:dark:border-neutral-700 hover:dark:bg-neutral-800/30 flex flex-col items-center justify-center"
              target="_blank"
              rel="noopener noreferrer"
              onClick={(e) => {
                e.preventDefault()
                handlerIncreaseWithEth()
              }}
            >
              <h2 className={`mb-3 text-2xl font-semibold`}>
                Increase
                <span className="inline-block transition-transform group-hover:translate-x-1 motion-reduce:transform-none">
                  -&gt;
                </span>
              </h2>
              <p className={`m-0 max-w-[30ch] text-sm opacity-50`}>
                {eth.hasProvider ? 'With eth wallet' : 'Please install the wallet first'}
              </p>
            </a>
          </div>
        </>
      )}
    </main>
  )
}
