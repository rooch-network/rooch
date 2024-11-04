import { FC, useEffect, useState } from 'react'
import { Button, Cell, Section } from '@telegram-apps/telegram-ui'
import './ConnectRoochPage.css'
import {Args, BitcoinAddress, fromHEX, Session, toB64} from '@roochnetwork/rooch-sdk'
import { useTonConnectUI } from '@tonconnect/ui-react'
import { TonProofItemReplySuccess } from '@tonconnect/ui-react'
import { openLink } from '@telegram-apps/sdk-react'
import { useNetworkVariable } from '@/use-networks.ts'
import {useRoochClient} from "@roochnetwork/rooch-sdk-kit";
interface ConnectRoochPageProps {
  onConnectSuccess?: () => void;
}
export const ConnectRoochPage: FC<ConnectRoochPageProps> = ({onConnectSuccess}) => {
  const [first, setFirst] = useState(true)
  const [btcAddr, setBtcAddr] = useState<string>('bc1q04uaa0mveqtt4y0sltuxtauhlyl8ctstr5x3hu')
  const [error, setError] = useState<string | undefined>()
  const [tonUI] = useTonConnectUI()
  const [proof, setProof] = useState<TonProofItemReplySuccess | undefined>()
  const counterContract = useNetworkVariable('counterContract')
  const [session, setSession] = useState<Session | undefined>()
  const portal = useNetworkVariable('portal')
  const client = useRoochClient()

  const onBtcAddrChanged = (event: React.ChangeEvent<HTMLInputElement>) => {
    setBtcAddr(event.target.value)
    setError(undefined)
  }
  const createSession = () => {
    if (proof) {
      return
    }
    if (!btcAddr) {
      setError('Please enter the btc address')
      return
    }

    try {
      new BitcoinAddress(btcAddr)
    } catch (e) {
      setError('Invalid btc address')
    }

    const payload = { tonProof: btcAddr }
    tonUI.setConnectRequestParameters({ state: 'ready', value: payload })
  }

  const buildSession = () => {
    if (session) {
      return
    }
    const _session = Session.Build({
      appName: 'rooch_test',
      appUrl: 'https://test.com',
      scopes: [`${counterContract}::*::*`],
      addr: new BitcoinAddress(btcAddr)
    })

    const decoder = new TextEncoder();
    const jsonInfo = decoder.encode(JSON.stringify(_session.toJSON()));
    const authUrl = `${portal}${toB64(jsonInfo)}`
    setSession(session)
    openLink(authUrl)
  }

  useEffect(() => {
    if (first) {
      setFirst(false)
      tonUI.setConnectRequestParameters({ state: 'loading' })
    }

    tonUI.onStatusChange(async (wallet) => {
      if (!wallet) {
        return
      }
      if (wallet.connectItems?.tonProof && 'proof' in wallet.connectItems.tonProof) {
        setProof(wallet.connectItems.tonProof)
        buildSession()
      }
    })
  }, [tonUI])

  useEffect(() => {
    if (session) {
      let pulling = false
      const interval = setInterval(async () => {
        if (pulling) {
          return
        }

        pulling = true
        const result = await client.executeViewFunction({
          target: '0x3::session_key::exists_session_key',
          args: [Args.address(''), Args.vec('u8', Array.from(fromHEX(session.getAuthKey())))]
        })

        if (result.return_values && result.return_values[0].decoded_value === true) {
          localStorage.setItem('session-key', session.toJSON())
          clearInterval(interval)
          if (onConnectSuccess) {
            onConnectSuccess()
          }
        }
      }, 1000)

      return () => clearInterval(interval)
    }
  }, [])
  return (
    <Section header="Connect Rooch" footer="Bind BTC Address & Create Rooch session-key">
      <Cell subtitle={error ? error : 'Input your btc address'}>
        <input
          type="text"
          value={btcAddr}
          className="input-text"
          onChange={onBtcAddrChanged}
          placeholder="Enter Bitcoin Address"
        />
      </Cell>
      <Cell
        style={{
          display: 'flex',
          justifyContent: 'center',
          width: '100%',
        }}
      >
        <Button onClick={createSession}>{proof ? 'con' : 'Create'}</Button>
      </Cell>
      <Cell
        style={{
          display: 'flex',
          justifyContent: 'center',
          width: '100%',
        }}
      >
        <Button
          onClick={() => {
            tonUI.disconnect()
          }}
        >
          Dis
        </Button>
      </Cell>
    </Section>
  )
}
