// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  useConnectWallet,
  useCreateSessionKey,
  useWalletStore,
  useCurrentSession,
  useWallets
} from '@roochnetwork/rooch-sdk-kit';
import {Button} from '@radix-ui/themes';

import './App.css'
import {useEffect, useState} from 'react';

export function App() {

  const account = useWalletStore((state) => state.currentAccount)
  const connectionStatus = useWalletStore((state) => state.connectionStatus)
  const sessionAccount = useCurrentSession()
  const {mutateAsync: connectWallet} = useConnectWallet()
  const wallets = useWallets()
  const installedWallets = wallets.filter((w) => w.installed === true)
  const [roochAddress, setRoochAddress] = useState('');

  useEffect(() => {
    async function fetchData() {
      try {
        const address = await sessionAccount?.getRoochAddress();
        setRoochAddress(address || '');
      } catch (error) {
        // 处理错误
        console.error('Failed to fetch Rooch address:', error);
      }
    }

    fetchData();
  }, [sessionAccount]);

  return (
    <div className="App">
      <p> Wallet Demo</p>

      {connectionStatus === 'connected' ? (
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
          }}
        >
          <div
            title="Basic Info"
            style={{width: 300, margin: 10}}
          >
            <div style={{textAlign: 'left', marginTop: 10}}>
              <div style={{fontWeight: 'bold'}}>Address:</div>
              <div style={{wordWrap: 'break-word'}}>{account?.address}</div>
            </div>

            <div style={{textAlign: 'left', marginTop: 10}}>
              <div style={{fontWeight: 'bold'}}>PublicKey:</div>
              <div style={{wordWrap: 'break-word'}}>{account?.publicKey}</div>
            </div>

            <div style={{textAlign: 'left', marginTop: 10}}>
              <div style={{fontWeight: 'bold'}}>Compressed PublicKey:</div>
              <div style={{wordWrap: 'break-word'}}>{account?.compressedPublicKey}</div>
            </div>
            <div style={{textAlign: 'left', marginTop: 10}}>
              <div style={{fontWeight: 'bold'}}>mul address:</div>
              <div style={{wordWrap: 'break-word'}}>{account?.toMultiChainAddress()?.toBech32()}</div>
            </div>
            <div style={{textAlign: 'left', marginTop: 10}}>
              <div style={{fontWeight: 'bold'}}>Session Account Address:</div>
              <div style={{wordWrap: 'break-word'}}>{roochAddress}</div>
            </div>
          </div>
          <CreateSessionCard/>
        </div>
      ) : (
        <div>
          {
            installedWallets.length === 0 ? 'Please install the wallet and try again' : connectionStatus !== 'disconnected' ? connectionStatus :
              <Button
                style={{marginTop: 10}}
                onClick={async () => {
                  await connectWallet({
                    wallet: installedWallets[0]
                  })
                }}>
                Connect Wallet
              </Button>
          }
        </div>
      )}
    </div>
  )

  function CreateSessionCard() {
    const {mutate: createSessionKey} = useCreateSessionKey()
    return (
      <div title="creating session key" style={{width: 300, margin: 10}}>
        <Button
          style={{marginTop: 10}}
          onClick={async () => {

            createSessionKey(
              {},
              {
                onSuccess: (result) => {
                  console.log('session key', result);
                },
              },
            );
          }}
        >create session key</Button>
      </div>
    )
  }
}