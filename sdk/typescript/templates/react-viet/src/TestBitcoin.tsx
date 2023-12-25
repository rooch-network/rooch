import { useEffect, useRef, useState } from "react"
import { useRoochClient } from "@roochnetwork/rooch-sdk-kit"
import { Account, IAuthorization, IAuthorizer, HexString } from "@roochnetwork/rooch-sdk"
import { sha3_256 } from "@noble/hashes/sha3"

import "./App.css"
import { Button } from "@radix-ui/themes";

function App() {

  const [unisatInstalled, setUnisatInstalled] = useState(false)
  const [connected, setConnected] = useState(false)
  const [accounts, setAccounts] = useState<string[]>([])
  const [publicKey, setPublicKey] = useState("")
  const [address, setAddress] = useState("")
  const [balance, setBalance] = useState({
    confirmed: 0,
    unconfirmed: 0,
    total: 0,
  })

  const getBasicInfo = async () => {
    const unisat = (window as any).unisat
    const [address] = await unisat.getAccounts()
    setAddress(address)

    console.log(address)
    const publicKey = await unisat.getPublicKey()
    setPublicKey(publicKey)

    console.log(publicKey)

    const balance = await unisat.getBalance()
    setBalance(balance)

  }

  const selfRef = useRef<{ accounts: string[] }>({
    accounts: [],
  })
  const self = selfRef.current
  const handleAccountsChanged = (_accounts: string[]) => {
    if (self.accounts[0] === _accounts[0]) {
      // prevent from triggering twice
      return
    }
    self.accounts = _accounts
    if (_accounts.length > 0) {
      setAccounts(_accounts)
      setConnected(true)

      setAddress(_accounts[0])

      getBasicInfo()
    } else {
      setConnected(false)
    }
  }

  const handleNetworkChanged = (_: string) => {
    getBasicInfo()
  }

  useEffect(() => {

    async function checkUnisat() {
      let unisat = (window as any).unisat

      for (let i = 1 ;i < 10 && !unisat; i += 1) {
        await new Promise((resolve) => setTimeout(resolve, 100*i))
        unisat = (window as any).unisat
      }

      if(unisat){
        setUnisatInstalled(true)
      }else if (!unisat)
        return

      unisat.getAccounts().then((accounts: string[]) => {
        handleAccountsChanged(accounts)
      })

      unisat.on("accountsChanged", handleAccountsChanged)
      unisat.on("networkChanged", handleNetworkChanged)

      return () => {
        unisat.removeListener("accountsChanged", handleAccountsChanged)
        unisat.removeListener("networkChanged", handleNetworkChanged)
      }
    }

    checkUnisat().then()
  }, [])

  if (!unisatInstalled) {
    return (
      <div className="App">
        <header className="App-header">
          <div>
            <Button
              onClick={() => {
                window.location.href = "https://unisat.io"
              }}
            >
              Install Unisat Wallet
            </Button>
          </div>
        </header>
      </div>
    )
  }
  const unisat = (window as any).unisat
  return (
    <div className="App">
      <header className="App-header">
        <p>Unisat Wallet Demo</p>

        {connected ? (
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
                <div style={{ wordWrap: "break-word" }}>{address}</div>
              </div>

              <div style={{ textAlign: "left", marginTop: 10 }}>
                <div style={{ fontWeight: "bold" }}>PublicKey:</div>
                <div style={{ wordWrap: "break-word" }}>{publicKey}</div>
              </div>

              <div style={{ textAlign: "left", marginTop: 10 }}>
                <div style={{ fontWeight: "bold" }}>Balance: (Satoshis)</div>
                <div style={{ wordWrap: "break-word" }}>{balance.total}</div>
              </div>
            </div>

            <SignMessageCard accounts = {accounts} publicKey = {publicKey}/>
          </div>
        ) : (
          <div>
            <Button
              onClick={async () => {
                const result = await unisat.requestAccounts()
                handleAccountsChanged(result)
              }}
            >
              Connect Unisat Wallet
            </Button>
          </div>
        )}
      </header>
    </div>
  )
}

function base64ToUint8Array(base64String: string) {
  // Step 1: Decode Base64 to binary string
  const binaryString = atob(base64String);

  // Step 2: Convert binary string to Uint8Array
  const uint8Array = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    uint8Array[i] = binaryString.charCodeAt(i);
  }

  return uint8Array;
}

function uint8ArrayToBase64(uint8Array: Uint8Array): string {
  let binaryString = '';
  uint8Array.forEach(byte => {
    binaryString += String.fromCharCode(byte);
  });

  return btoa(binaryString);
}

class TestAuth implements IAuthorizer {
  private publicKey: string

  constructor(publicKey: string) {
    this.publicKey = publicKey
  }

  async auth(callData: Uint8Array): Promise<IAuthorization> {

    console.log('需要签名的数据 Array, ', callData, ' len = ', callData.length)

    const msgHash = sha3_256(callData)

    console.log('需要签名的数据 Hash-Array, ', msgHash, ' len = ', msgHash.length)

    const msgHashHex = Array.from(msgHash).map(b => b.toString(16).padStart(2, '0')).join('')

    console.log('需要签名的数据 Hash-Hex, ', msgHashHex, ' len = ', msgHash.length)

    const signature = await (window as any).unisat.signMessage(msgHashHex)

    console.log('签名 Base64 ', signature, ' len = ', signature.length)

    const binaryData = atob(signature)

    console.log('签名 Array', binaryData, ' len = ', binaryData.length)

    let signHexString = ""
    for (let i = 0; i < binaryData.length; i++) {
      const hex = binaryData.charCodeAt(i).toString(16).padStart(2, '0')
      signHexString += hex
    }

    console.log('签名 Hex', signHexString, ' len = ', signHexString.length)

    const signUint8Array = base64ToUint8Array(signature)

    console.log('sign, arr', signUint8Array)

    // test recover base64
    const signBase64 = uint8ArrayToBase64(signUint8Array)
    if (signBase64 !== signature) {
      console.log('sign 编解码不正确')
      throw ''
    }

    const publicKeyArray = new HexString(this.publicKey).toUint8Array()

    console.log('pubUint8', publicKeyArray, publicKeyArray.length)

    const payload = new Uint8Array(publicKeyArray.length + signUint8Array.length)
    payload.set(signUint8Array)
    payload.set(publicKeyArray, signUint8Array.length)

    const auth :IAuthorization= {
      scheme: 2,
      payload: payload,
    }

    console.log('auth', auth)

    return auth
  }
}

function SignMessageCard(props:any) {
  const rooch = useRoochClient()

  // const devCounterAddress = '0x49ee3cf17a017b331ab2b8a4d40ecc9706f328562f9db63cba625a9c106cdf35'
  // const devCounterModule = `${devCounterAddress}::counter`

  const [signature, setSignature] = useState("")

  return (
    <div title="Sign Message" style={{ width: 300, margin: 10 }}>
      <div style={{ textAlign: "left", marginTop: 10 }}>
        <div style={{ fontWeight: "bold" }}>Message:</div>
      </div>
      <div style={{ textAlign: "left", marginTop: 10 }}>
        <div style={{ fontWeight: "bold" }}>Signature:</div>
        <div style={{ wordWrap: "break-word" }}>{signature}</div>
      </div>
      <Button
        style={{ marginTop: 10 }}
        onClick={async () => {

          const roochAddress = await rooch.resoleRoochAddress(props.accounts[0])

          const defaultScope = [
            '0x1::*::*',
            '0x3::*::*',
            '0x49ee3cf17a017b331ab2b8a4d40ecc9706f328562f9db63cba625a9c106cdf35::*::*',
          ]

          const authorizer = new TestAuth(props.publicKey)
          const account = new Account(rooch, roochAddress, authorizer)

          const s = await account.createSessionAccount(defaultScope, 1200)

          console.log(s)

          console.log('尝试调用 coutner')
          // const func = `${devCounterModule}::increase`

          // const result = await s?.runFunction(func, [], [], { maxGasAmount: 10000 })

          // console.log(result)

          setSignature(signature)
        }}
      >
        Sign Message
      </Button>
    </div>
  )
}

export default App


