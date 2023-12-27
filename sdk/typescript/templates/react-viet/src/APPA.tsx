// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  useCreateSessionKey,
  useCurrentSessionAccount,
  useWalletStore
} from "@roochnetwork/rooch-sdk-kit";
import { Button } from "@radix-ui/themes";

import "./App.css"

export function App() {

  const account = useWalletStore((state) => state.currentAccount)
  const connectionStatus = useWalletStore((state) => state.connectionStatus)
  const sessionAccount = useCurrentSessionAccount()

  return (
    <div className="App">
        <p> Wallet Demo</p>

        {connectionStatus === 'connected' ? (
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
            }}
          >
            <div
              title="Basic Info"
              style={{ width: 300, margin: 10 }}
            >
              <div style={{ textAlign: "left", marginTop: 10 }}>
                <div style={{ fontWeight: "bold" }}>Address:</div>
                <div style={{ wordWrap: "break-word" }}>{account?.getAddress()}</div>
              </div>

              <div style={{ textAlign: "left", marginTop: 10 }}>
                <div style={{ fontWeight: "bold" }}>PublicKey:</div>
                <div style={{ wordWrap: "break-word" }}>{account?.getInfo().publicKey}</div>
              </div>

              <div style={{ textAlign: "left", marginTop: 10 }}>
                <div style={{ fontWeight: "bold" }}>Compressed PublicKey: </div>
                <div style={{ wordWrap: "break-word" }}>{account?.getInfo().compressedPublicKey}</div>
              </div>
              <div style={{ textAlign: "left", marginTop: 10 }}>
                <div style={{ fontWeight: "bold" }}>Session Account Address: </div>
                <div style={{ wordWrap: "break-word" }}>{sessionAccount?.getAddress()}</div>
              </div>
            </div>
            <CreateSessionCard/>
          </div>
        ) : (
          <div>
            {connectionStatus}
          </div>
        )}
    </div>
    )

  function CreateSessionCard() {
    const { mutate: createSessionKey} = useCreateSessionKey()
    return (
      <div title="creating session key" style={{ width: 300, margin: 10 }}>
        <div style={{ textAlign: "left", marginTop: 10 }}>
          {/*<div style={{ fontWeight: "bold" }}>session key: {sessionKey.data?.getAddress()}</div>*/}
        </div>
        <Button
          style={{ marginTop: 10 }}
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

  // function senTxCard() {
  //   const devCounterAddress = '0x49ee3cf17a017b331ab2b8a4d40ecc9706f328562f9db63cba625a9c106cdf35'
  //   const devCounterModule = `${devCounterAddress}::counter`
  //
  //   const sessionKey = useSessionKey()
  //
  //   console.log('sessionKey', sessionKey)
  //
  //   return (
  //     <div title="Sign Message" style={{ width: 300, margin: 10 }}>
  //       <div style={{ textAlign: "left", marginTop: 10 }}>
  //         <div style={{ fontWeight: "bold" }}>Message:</div>
  //       </div>
  //       <div style={{ textAlign: "left", marginTop: 10 }}>
  //         <div style={{ fontWeight: "bold" }}>Signature:</div>
  //       </div>
  //       <Button
  //         style={{ marginTop: 10 }}
  //         onClick={async () => {
  //           // local need deploy counter contract
  //           const func = `${devCounterModule}::increase`
  //
  //           const result = await sessionKey?.runFunction(func, [], [], { maxGasAmount: 10000 })
  //
  //           console.log(result)
  //         }}
  //       >
  //         Sign Message
  //       </Button>
  //     </div>
  //   )
  // }
}