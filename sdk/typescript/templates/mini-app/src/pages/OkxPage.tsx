import { FC, useEffect, useState } from 'react'

import { Page } from '@/components/Page.tsx'
import { Cell, Image, Section } from '@telegram-apps/telegram-ui'
import roochSvg from '@/pages/rooch_white_logo.svg'
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
    // const okxTonConnectUI = new OKXTonConnectUI({
    //     dappMetaData: {
    //         name: "application name",
    //         icon: "application icon url"
    //     },
    //     actionsConfiguration:{
    //         returnStrategy:'none',
    //         tmaReturnUrl:'back'
    //     },
    //     uiPreferences: {
    //         theme: THEME.LIGHT
    //     },
    //     language: 'en_US',
    //     restoreConnection: true
    // });
    //
    // setOkx(okxTonConnectUI)
  }, [])

  return (
    <Page>
      <Section
        header="Features"
        footer="You can use these pages to learn more about features, provided by Rooch"
      >
        <Cell
          before={
            <Image
              src={roochSvg}
              style={{ backgroundColor: '#000000' }}
              onClick={async () => {
                try {
                  console.log('connect', okxProvider)
                  try {
                    const session = await okxProvider?.connect({
                      namespaces: {
                        btc: {
                          chains: ['btc:mainnet'],
                        },
                      },
                      sessionConfig: {
                        redirect: 'tg://resolve',
                      },
                    })
                    console.log('connect ok', session)
                  } catch (e) {
                    console.log(e)
                  }
                } catch (e) {
                  console.log(e)
                }
              }}
            />
          }
          subtitle="Connect OKX"
        >
          connect
        </Cell>
        {/*</a>*/}
      </Section>
    </Page>
  )
}
