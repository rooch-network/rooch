import { useState } from 'react'
import { hexlify } from '@ethersproject/bytes'
import MetaMaskSDK from '@metamask/sdk'
import { encodeMoveCallData } from '@rooch/sdk'

import './App.css'

const DEFAULT_COUNTER_ADDRESS = '0x95b15595f939a8cf5ece25d44fc832acaf218a6028472b9c0785338fae21bef9'
const ROOCH_ADDRESS = '0xd46e8dd67c5d32be8058bb8eb970870f07244568' // TODO: can be fixed rooch address

const options = {
  dappMetadata: {},
}

const MMSDK = new MetaMaskSDK(options)
const ethereum = MMSDK.getProvider()
const counterAddress =
  new URLSearchParams(window.location.search).get('counter_address') || DEFAULT_COUNTER_ADDRESS

function App() {
  const [counter, setCounter] = useState<number>(1)

  const handleClick = async () => {
    if (!ethereum) {
      console.error('Ethereum provider is not available.')
      return
    }

    const accounts = (await ethereum.request({
      method: 'eth_requestAccounts',
      params: [],
    })) as string[]
    if (!accounts) {
      console.error('No accounts available.')
      return
    }

    const moveCallData = encodeMoveCallData(`${counterAddress}::counter::increase`, [], [])

    const params = [
      {
        from: accounts[0],
        to: ROOCH_ADDRESS,
        gas: '0x76c0', // 30400
        gasPrice: '0x9184e72a000', // 10000000000000
        value: '0x4e72a', // 2441406250
        data: hexlify(moveCallData),
      },
    ]

    try {
      await ethereum.request({
        method: 'eth_sendTransaction',
        params,
      })

      setCounter((v) => v + 1)
    } catch (e: any) {
      console.error('Error occurred while sending transaction:', e)
      alert(e.message)
    }
  }

  return (
    <div className="App">
      <header className="App-header">
        <div>
          <div>Counter: {counter}</div>
          <button className="Inc-Btn" onClick={handleClick}>
            Inc
          </button>
        </div>
      </header>
    </div>
  )
}

export default App
