// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Decimal } from 'decimal.js'
import { useEffect, useState } from 'react'
import { Args, fixedBalance } from '@roochnetwork/rooch-sdk'

import * as styles from './SwapGasView.css.js'
import { MarketEev } from '../env.js'
import { View } from '../../ui/View.js'
import { Input } from '../../ui/Input.js'
import { Card } from '../../ui/Card.js'
import { Heading } from '../../ui/Heading.js'
import { Text } from '../../ui/Text.js'
import { BitcoinIcon } from '../../icons/BitcoinIcon.js'
import { RGasIcon } from '../../icons/RGasIcon.js'
import { useProgress } from '../../ProgressProvider.js'
import { useDebounce } from '../../util/debounce.js'

import {
  useCurrentAddress,
  useCurrentNetwork,
  useCurrentWallet,
  useRoochClient,
} from '../../../hooks/index.js'

const btcDecimals = 100000000

export function SwapGasView() {
  const { wallet } = useCurrentWallet()
  const currentAddress = useCurrentAddress()
  const client = useRoochClient()
  const network = useCurrentNetwork()
  const env = network === 'mainnet' ? MarketEev.main : MarketEev.test
  const [rate, setRate] = useState(0)
  const [to, setTo] = useState(0)
  const [rGasBalance, setRGasBalance] = useState(0)
  const [bitcoinBalance, setBitcoinBalance] = useState(0)
  const [form, setForm] = useState('0.0')
  const debouncedForm = useDebounce(form, 500)
  const { start, finish } = useProgress()
  const [errorMessage, setErrorMessage] = useState<string>()

  useEffect(() => {
    if (!currentAddress) {
      return
    }

    const fetchBalances = async () => {
      try {
        const bitcoinResult = await wallet?.getBalance()
        const rGasResult = await client.getBalance({
          owner: currentAddress,
          coinType: '0x3::gas_coin::RGas',
        })

        setBitcoinBalance(bitcoinResult?.confirmed || 0)
        setRGasBalance(rGasResult.fixedBalance)

        finish()
      } catch (error) {
        setErrorMessage('Failed to fetch balances. Please try again later.')
      }
    }
    start()
    fetchBalances().finally(() => finish())
  }, [client, currentAddress, wallet, start, finish])

  useEffect(() => {
    const fromNumber = Number(debouncedForm)
    if (fromNumber <= 0) {
      return
    }

    start()
    const stas = fromNumber * btcDecimals
    client
      .executeViewFunction({
        address: env.marketCA,
        module: 'gas_market',
        function: 'btc_to_rgas',
        args: [Args.u64(BigInt(stas))],
      })
      .then((result) => {
        const value = Number(result.return_values?.[0]?.decoded_value || 0) || 0
        const s = new Decimal(value).div(stas.toString()).toNumber()
        setTo(value)
        setRate(s)
      })
      .finally(() => finish())
  }, [client, finish, debouncedForm, env.marketCA, start])

  const swap = () => {
    start()
    wallet
      ?.sendBtc({
        toAddress: env.btcGasAddress,
        satoshis: Number(Number(debouncedForm) * btcDecimals),
      })
      .finally(() => finish())
  }

  return (
    <View title="Swap Gas" actionText="Swap" actionOnClick={swap}>
      <div className={styles.container}>
        <Card header="From" headerRight={`Balance: ${fixedBalance(bitcoinBalance, 9)}`}>
          <div className={styles.inputContainer}>
            <Input
              type="number"
              value={form}
              onChange={(v) => {
                const value = v.target.value
                // Use a regular expression to allow only whole numbers
                if (/^\d*\.?\d*$/.test(value)) {
                  setForm(value)
                }
              }}
            />
            <BitcoinIcon />
          </div>
        </Card>
        <Card header="To" headerRight={`Balance: ${rGasBalance}`}>
          <div className={styles.inputContainer}>
            <Input
              disabled={true}
              value={Intl.NumberFormat('en-us').format(Number(fixedBalance(to, 8).toFixed(2)))}
            />
            <RGasIcon />
          </div>
        </Card>
      </div>
      <div className={styles.infoContainer}>
        <Heading as="h3" size="sm" weight="normal">
          {errorMessage}
          {!errorMessage &&
            `1 BTC = ${rate === 0 ? '♾️' : Intl.NumberFormat('en-us').format(rate)} RGas`}
        </Heading>
        <div className={styles.separator} />
        <Text weight="medium" color="muted">
          Estimated rate, the actual amount received depends on the rate at the time of transaction
          confirmation block.
        </Text>
      </div>
    </View>
  )
}
