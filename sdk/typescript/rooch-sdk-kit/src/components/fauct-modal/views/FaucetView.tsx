// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Heading } from '../../ui/Heading.js'
import * as styles from './FaucetView.css.js'
import { Text } from '../../ui/Text.js'
import { Button } from '../../ui/Button.js'
import { useCallback, useEffect, useState } from 'react'
import {
  useCurrentAddress,
  useCurrentNetwork,
  useCurrentWallet,
  useRoochClient,
} from '../../../hooks/index.js'
import { ENVS } from '../env.js'
import { Args, fixedBalance, stringToBytes, toHEX } from '@roochnetwork/rooch-sdk'
import { useTriggerRequest } from '../../../provider/globalProvider.js'
import { useProgress } from '../../ProgressProvider.js'

const FAUCET_NOT_OPEN = "Oops! The faucet isn't open right now. 🚰🔒 Check back later!"
const INVALID_UTXO = "Hmm, something's not right with the UTXO. 🤔🔍 Maybe double-check it?"
const FAUCET_NOT_ENOUGH_RGAS = "Looks like we're running low on RGas. ⛽️💨 Please try again soon!"
const ALREADY_CLAIMED = "You've already claim your RGas! 🎉😎 Enjoy!"
const UTXO_VALUE_IS_ZERO = "The UTXO value is zero. 💸🧐 Please ensure it's got some value!"
const unknowError = 'An unknown error occurred. Please try again later. 🤷‍♂️'

const ERROR_MSG: Record<string, string> = {
  1: FAUCET_NOT_OPEN,
  2: INVALID_UTXO,
  3: FAUCET_NOT_ENOUGH_RGAS,
  4: ALREADY_CLAIMED,
  5: UTXO_VALUE_IS_ZERO,
}

type FaucetViewProps = {
  inviter?: string
  swapRGas: () => void
}

export function FaucetView({ inviter, swapRGas }: FaucetViewProps) {
  const client = useRoochClient()
  const { wallet } = useCurrentWallet()
  const currentNetwork = useCurrentNetwork()
  const currentAddress = useCurrentAddress()
  const [loading, setLoading] = useState<string>()
  const { start, finish } = useProgress()
  const [faucetAward, setFaucetAward] = useState(0)
  const [errorMessage, setErrorMessage] = useState<string>()
  const [claimedMessage, setClaimedMessage] = useState<string>()
  const [needCheck, setNeedCheck] = useState<boolean>(false)
  const triggerRequest = useTriggerRequest()

  const env = currentNetwork === 'mainnet' ? ENVS.main : ENVS.test

  const startLoading = useCallback(
    (msg: string) => {
      setLoading(msg)
      start()
    },
    [start],
  )

  const finishLoading = useCallback(() => {
    finish(() => {
      setLoading(undefined)
    })
  }, [finish])

  const checkClaim = useCallback(() => {
    if (!currentAddress) {
      return
    }
    setFaucetAward(0)
    setErrorMessage(undefined)
    setClaimedMessage(undefined)
    startLoading('Verifying your eligibility for gas... 🕵️‍♀️✨')

    client
      .queryUTXO({
        filter: {
          owner: currentAddress.toStr(),
        },
      })
      .then(async (result) => {
        const utxoIds = result.data.map((item) => item.id)
        if (utxoIds.length > 0) {
          const result = await client.executeViewFunction({
            target: `${env.faucet.CA}::gas_faucet::check_claim`,
            args: [
              Args.objectId(env.faucet.Obj),
              Args.address(currentAddress.genRoochAddress().toHexAddress()),
              Args.vec('objectId', utxoIds),
            ],
          })

          if (result.vm_status === 'Executed') {
            const gas = Number(fixedBalance(Number(result.return_values![0].decoded_value), 8))
            setFaucetAward(gas)
          } else if ('MoveAbort' in result.vm_status) {
            setErrorMessage(ERROR_MSG[Number(result.vm_status.MoveAbort.abort_code)])
          }
        } else {
          setErrorMessage(INVALID_UTXO)
        }
      })
      .catch((e: any) => {
        console.log(e)
        setErrorMessage(unknowError)
      })
      .finally(() => {
        finishLoading()
        setNeedCheck(false)
      })
  }, [client, currentAddress, env.faucet.CA, env.faucet.Obj, finishLoading, startLoading])

  useEffect(() => {
    checkClaim()
  }, [checkClaim])

  const handleClaim = async () => {
    if (errorMessage) {
      swapRGas()
      return
    }

    try {
      startLoading('Hang tight, your RGas is zooming your way! 🚀✨')

      const response = await (inviter ? claimWithInviter() : claim())

      if (!response.ok) {
        const data = await response.json()
        if (response.status === 500 && data.error.includes('UTXO value is zero')) {
          setErrorMessage(ALREADY_CLAIMED)
          return
        }
        setErrorMessage(unknowError)
        return
      }

      const d: any = await response.json()
      const awardRGAS = fixedBalance(d.gas || 0, 8)
      setClaimedMessage(`Awesome! You've got ${awardRGAS} RGas in your pocket! 🚀🎉`)
      setNeedCheck(true)
      triggerRequest('success')
    } catch (e: any) {
      if ('message' in e) {
        setErrorMessage(e.message)
      } else {
        setErrorMessage(unknowError)
      }
    } finally {
      finishLoading()
    }
  }

  const claim = () => {
    const payload = JSON.stringify({
      claimer: currentAddress?.toStr(),
    })
    return fetch(`${env.faucet.Url}/faucet`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: payload,
    })
  }

  const claimWithInviter = async () => {
    // step 1 check inviter
    const result = await client.queryObjectStates({
      filter: {
        object_type: `${env.inviter.CA}::${env.inviter.Module}::${env.inviter.Conf}`,
      },
    })

    if (result && result.data.length > 0 && result.data[0].decoded_value?.value.is_open === true) {
      const pk = wallet!.getPublicKey().toBytes()
      const signMsg = 'Welcome to use Rooch! Hold BTC Claim your RGas.'
      const sign = await wallet!.sign(stringToBytes('utf8', signMsg))

      const payload = JSON.stringify({
        claimer: currentAddress?.toStr(),
        inviter: inviter,
        claimer_sign: toHEX(sign),
        public_key: toHEX(pk),
        message: signMsg,
      })

      return await fetch(`${env.faucet.Url}/faucet-inviter`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: payload,
      })
    } else {
      return claim()
    }
  }

  return (
    <div className={styles.container}>
      <Heading as="h2">Rooch Faucet</Heading>
      <div className={styles.content}>
        <Text weight="medium" color="muted">
          {loading}
          {!loading && errorMessage}
          {!loading && claimedMessage}
          {!loading && !claimedMessage && faucetAward
            ? `Yay! You can claim ${faucetAward} RGas! 🎉💧`
            : undefined}
        </Text>
      </div>
      <div className={styles.createButtonContainer}>
        <Button
          disabled={loading !== undefined}
          type="button"
          variant="outline"
          onClick={needCheck ? checkClaim : handleClaim}
        >
          {errorMessage ? 'Swap RGas' : needCheck ? 'Check' : 'Claim'}
        </Button>
      </div>
    </div>
  )
}
