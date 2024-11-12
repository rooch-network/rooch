import { FC, useEffect, useState } from 'react'
import { Button, Cell, Section } from '@telegram-apps/telegram-ui'
import './ConnectRoochPage.css'
// import {Args, BitcoinAddress, fromHEX, Session, toB64, Transaction} from '@roochnetwork/rooch-sdk'
import { useTonConnectUI } from '@tonconnect/ui-react'
// import { TonProofItemReplySuccess } from '@tonconnect/ui-react'
// import { openLink } from '@telegram-apps/sdk-react'
// import { useNetworkVariable } from '@/use-networks.ts'
// import {useRoochClient, useRoochClientQuery} from "@roochnetwork/rooch-sdk-kit";
interface ConnectRoochPageProps {
  onConnectSuccess?: () => void
}

export const ConnectRoochPage: FC<ConnectRoochPageProps> = ({ onConnectSuccess }) => {
  const [first, setFirst] = useState(true)
  const [btcAddr, setBtcAddr] = useState<string>('bc1q04uaa0mveqtt4y0sltuxtauhlyl8ctstr5x3hu')
  const [error, setError] = useState<string | undefined>()
  const [tonUI] = useTonConnectUI()
  // const [proof, setProof] = useState<TonProofItemReplySuccess | undefined>()
  // const counterContract = useNetworkVariable('counterContract')
  // const [session, setSession] = useState<Session | undefined>()
  // const portal = useNetworkVariable('portal')
  // const client = useRoochClient()
  // const [tx, setTx] = useState<Transaction | undefined>()

  console.log(onConnectSuccess)

  const onBtcAddrChanged = (event: React.ChangeEvent<HTMLInputElement>) => {
    setBtcAddr(event.target.value)
    setError(undefined)
  }

  const create2Proof = () => {
    const payload = { tonProof: btcAddr }
    tonUI.setConnectRequestParameters({ state: 'ready', value: payload })
    const payload1 = { tonProof: 'test-proof' }
    tonUI.setConnectRequestParameters({ state: 'ready', value: payload1 })
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
        console.log('proof ---- ')
        console.log(wallet.connectItems)
      }
    })
  }, [tonUI])

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
        <Button onClick={create2Proof}>create proof</Button>
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
