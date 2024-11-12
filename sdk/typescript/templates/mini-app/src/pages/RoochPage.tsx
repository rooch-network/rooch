import { FC, useState } from 'react'

import { Page } from '@/components/Page.tsx';
import {Cell, Image, Section} from '@telegram-apps/telegram-ui'
import roochSvg from '@/pages/rooch_white_logo.svg'
import { BitcoinAddress, Session, toB64, Transaction } from "@roochnetwork/rooch-sdk";
import { useRoochClient, useRoochClientQuery } from "../../../../rooch-sdk-kit";

const devCounterAddress = "0xf859b4113ddd951a694e2d5d3f5849e1ccd43b3cfef92ec8f8f8a46200d3df75"
const devCounterModule = `${devCounterAddress}::counter`
const defaultScopes = [
	`${devCounterAddress}::*::*`,
]
export const RoochPage: FC = () => {

  const client = useRoochClient()
	const [btcAddr, setBtcAddr] = useState<string>();

	const session = Session.Build({
		appName: "rooch_test",
		appUrl: "https://test.com",
		scopes: defaultScopes,
		addr: new BitcoinAddress('tb1qxvrzdqlnmpzxr6zsg7g2c62gu6l33qxzz6z5l2')
	})
  console.log(session.getAuthKey())
  const decoder = new TextEncoder();
  const jsonInfo = decoder.encode(JSON.stringify(session.toJSON()));
  const authUrl = `http://localhost:8083/session/${toB64(jsonInfo)}`

  console.log(authUrl)

  let {data, refetch} = useRoochClientQuery("executeViewFunction", {
    target: `${devCounterModule}::value`,
  })

  return (
    <Page>
      <Section
        header="Features"
        footer="You can use these pages to learn more about features, provided by Rooch"
      >
        {/*<a href="javascript:Telegram.WebApp.openLink('https://ton.org/');">Open link in external browser</a>*/}
        {/*<a href="http://localhost:8080">*/}
        <Cell subtitle="input your btc addr">
          <input
            type="text"
            value={btcAddr}
            onChange={(event: React.ChangeEvent<HTMLInputElement>) => setBtcAddr(event.target.value)}
            placeholder="Enter Bitcoin Address"
          />
        </Cell>
        <Cell
          before={<Image src={roochSvg} style={{ backgroundColor: '#000000' }} onClick={() => {
            console.log(authUrl)
          }}/>}
          subtitle="Connect Rooch"
        >
          connect
        </Cell>
        <Cell onClick={() => {
            try {
              const data = JSON.stringify({
                url: 'https://ton.org/',
                  try_instant_view: false
              });

              window.TelegramWebviewProxy?.postEvent('web_app_open_link', data)
            } catch (e) {
              console.log(e)
            }
        }}
          subtitle="auth url"
        >
          {authUrl}
        </Cell>
        <Cell
              subtitle="counter"
        >
          {data?.return_values? data.return_values[0].decoded_value as string: '0'}
        </Cell>
        <Cell onClick={async () => {
          const tx = new Transaction()
          tx.callFunction({
            target: `${devCounterModule}::increase`
          })

          const result = await client.signAndExecuteTransaction({
            transaction: tx,
            signer: session
          })

          if (result.execution_info.status.type !== 'executed') {
            console.log('increase failed')
          }

          refetch()
        }}
              subtitle="auth url"
        >
          increase
        </Cell>
        {/*</a>*/}
      </Section>
    </Page>
  )
};
