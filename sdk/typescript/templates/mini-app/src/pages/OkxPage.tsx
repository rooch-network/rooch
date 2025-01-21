import { FC, useEffect, useMemo, useState } from 'react'

import { Page } from '@/components/Page.tsx'
import { Cell, Image, Section } from '@telegram-apps/telegram-ui'
import roochSvg from './IndexPage/ton.svg'
import { OKXUniversalConnectUI, THEME } from '@okxconnect/ui'
import { OKXBtcProvider } from '@okxconnect/universal-provider'
import { useCreateSessionKey, useCurrentSession } from '@roochnetwork/rooch-sdk-kit'
import {
  Address,
  Authenticator,
  BitcoinAddress,
  BitcoinSignMessage,
  bytes,
  Bytes,
  fromHEX,
  PublicKey,
  RoochAddress,
  Secp256k1PublicKey,
  SignatureScheme,
  Signer,
  str,
  Transaction,
} from '@roochnetwork/rooch-sdk'

class CustomSigner extends Signer {
  btcProvider: OKXBtcProvider

  constructor(provider: OKXBtcProvider) {
    super()
    this.btcProvider = provider
  }

  sign(input: Bytes): Promise<Bytes> {
    console.log('sign')
    return new Promise(async (resolve, reject) => {
      try {
        const msgStr = str('utf8', input)
        const sign = await this.btcProvider.signMessage('btc:mainnet', msgStr)
        if (typeof sign !== 'string') {
          throw new Error('Method not implemented.')
        }
        const signedBytes = bytes('base64', sign as string).subarray(1)
        resolve(signedBytes)
      } catch (error) {
        reject(error)
      }
    })
  }
  signTransaction(input: Transaction): Promise<Authenticator> {
    const message = new BitcoinSignMessage(input.hashData(), input.getInfo() || '')
    return Authenticator.bitcoin(message, this, 'raw')
  }
  getBitcoinAddress(): BitcoinAddress {
    const address = this.btcProvider.getAccount('btc:mainnet')!.address
    return new BitcoinAddress(address)
  }
  getRoochAddress(): RoochAddress {
    return this.getBitcoinAddress().genRoochAddress()
  }
  getKeyScheme(): SignatureScheme {
    throw 'Secp256k1'
  }
  getPublicKey(): PublicKey<Address> {
    const publicKey = this.btcProvider.getAccount('btc:mainnet')!.publicKey
    return new Secp256k1PublicKey(fromHEX(publicKey))
  }
}

export const OkxPage: FC = () => {
  const [okxProvider, setOkxProvider] = useState<OKXUniversalConnectUI>()
  const [btcProvider, setBtcProvider] = useState<OKXBtcProvider>()
  const { mutate, error } = useCreateSessionKey()
  const session = useCurrentSession()

  console.log(error)

  useEffect(() => {
    if (okxProvider?.connected()) setBtcProvider(new OKXBtcProvider(okxProvider))
  }, [okxProvider])

  useEffect(() => {
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

  const handleConnect = async () => {
    try {
      console.log('connect', okxProvider)
      try {
        const session = await okxProvider?.openModal({
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
        setBtcProvider(new OKXBtcProvider(okxProvider!))
      } catch (e) {
        console.log(e)
      }
    } catch (e) {
      console.log(e)
    }
  }

  const handlerCreateSession = () => {
    if (btcProvider) {
      mutate({
        appName: 'test',
        appUrl: 'http://test.com',
        scopes: ['0x1::*::*'],
        signer: new CustomSigner(btcProvider),
      })
    }
  }

  console.log(okxProvider)
  btcProvider?.getAccount('btc:mainnet')

  return (
    <Page>
      <Section
        header="Features"
        footer="You can use these pages to learn more about features, provided by Rooch"
      >
        <Cell
          onClick={async () => handleConnect()}
          before={<Image src={roochSvg} style={{ backgroundColor: '#000000' }} />}
          subtitle={btcProvider ? 'Connected' : 'Connect'}
        >
          {btcProvider ? btcProvider.getAccount('btc:mainnet')?.address : ''}
        </Cell>

        <Cell onClick={handlerCreateSession}>
          {session ? session.getAuthKey() : 'Create Session'}
        </Cell>
      </Section>
    </Page>
  )
}
