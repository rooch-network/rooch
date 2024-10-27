import { FC, useState } from 'react'

import { Page } from '@/components/Page.tsx';
import {Cell, Image, Section} from '@telegram-apps/telegram-ui'
import roochSvg from '@/pages/rooch_white_logo.svg'
import { BitcoinAddress, Session } from '@roochnetwork/rooch-sdk'

const devCounterAddress = "0xf9b10e6c760f1cadce95c664b3a3ead3c985bbe9d63bd51a9bf1760785d26a1b"
const defaultScopes = [
	`${devCounterAddress}::*::*`,
]
export const RoochPage: FC = () => {

	const [btcAddr, setBtcAddr] = useState<string>();
	// const [authUrl, setAuthUrl] = useState<string>();

	const url = Session.Build({
		appName: "rooch_test",
		appUrl: "https://test.com",
		scopes: defaultScopes,
		addr: new BitcoinAddress('tb1qxvrzdqlnmpzxr6zsg7g2c62gu6l33qxzz6z5l2')
	})
  const authUrl = `http://localhost:8083?tg_session=${JSON.stringify(url)}`
  console.log(authUrl)
  console.log(url)

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
        <Cell onClick={() => {navigator.clipboard.writeText(authUrl);}}
          subtitle="auth url"
        >
          {authUrl}
        </Cell>
        {/*</a>*/}
      </Section>
    </Page>
  )
};
