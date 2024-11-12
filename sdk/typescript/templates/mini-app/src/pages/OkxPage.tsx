import { FC, useEffect, useState } from 'react'

import { Page } from '@/components/Page.tsx'
import { Cell, Image, Section } from '@telegram-apps/telegram-ui'
import tonSvg from './IndexPage/ton.svg'
import { OKXUniversalConnectUI, THEME } from '@okxconnect/ui'

export const OkxPage: FC = () => {
  const [okxProvider, setOkxProvider] = useState<OKXUniversalConnectUI>()

  useEffect(() => {
    // OKXUniversalProvider.init({
    //   dappMetaData: {
    //     icon: 'https://static.okx.com/cdn/assets/imgs/247/58E63FEA47A2B7D7.png',
    //     name: 'OKX Connect UI Demo',
    //   },
    // }).then((result) => {
    //   setOkxProvider(result)
    // })
    OKXUniversalConnectUI.init({
      dappMetaData: {
        icon: 'https://static.okx.com/cdn/assets/imgs/247/58E63FEA47A2B7D7.png',
        name: 'OKX WalletConnect UI Demo',
      },
      actionsConfiguration: {
        returnStrategy: 'tg://resolve',
        modals: 'all',
      },
      language: 'en_US',
      uiPreferences: {
        theme: THEME.LIGHT,
      },
    }).then((result) => {
      setOkxProvider(result)
    })
  }, [])

  return (
    <Page>
      <Section
        header="Features"
        footer="You can use these pages to learn more about features, provided by Rooch"
      >
        <Cell
          onClick={async () => {
            console.log('connect', okxProvider)
            try {
              const s = await okxProvider?.connect({
                namespaces: {
                  btc: {
                    chains: ['btc:mainnet'],
                  },
                },
                sessionConfig: {
                  redirect: 'tg://resolve',
                },
              })
              console.log('connectr', s)
            } catch (e) {
              console.log(e)
            }
          }}
          before={<Image src={tonSvg} style={{ backgroundColor: '#000000' }} />}
          subtitle="Connect Rooch"
        >
          connect
        </Cell>
      </Section>
    </Page>
  )
}
