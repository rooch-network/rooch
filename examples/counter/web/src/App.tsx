import { useState } from "react";
import MetaMaskSDK from '@metamask/sdk';
import {  hexlify } from '@ethersproject/bytes';
import { toUtf8Bytes } from '@ethersproject/strings';
import { encodeStructTypeTags, encodeFunctionCall } from './utils/tx';
import { BcsSerializer } from "./lib/runtime/bcs/mod";

import './App.css';

const options = {
  dappMetadata:{},
};
const MMSDK = new MetaMaskSDK(options);
const ethereum = MMSDK.getProvider(); // You can also access via window.ethereum

function App() {
  const [counter, setCounter] = useState<number>(1)

  const encodeMoveCallData = (functionId:string, tyArgs: string[], args:string[])=>{
    const encodedTypeArgs = encodeStructTypeTags(tyArgs)
    const encodedArgs = args.map((arg)=> toUtf8Bytes(arg))
    const scriptFunction = encodeFunctionCall(functionId, encodedTypeArgs, encodedArgs)

    // Multiple BcsSerializers should be used in different closures, otherwise, the latter will be contaminated by the former.
    const payloadInHex = (function () {
      const se = new BcsSerializer()
      scriptFunction.serialize(se)
      return hexlify(se.getBytes())
    })()

    return payloadInHex;
  }

  const handleClick = async ()=>{
    if (!ethereum){
      return
    }

    const accounts = await ethereum.request({ method: 'eth_requestAccounts', params: [] }) as string[];
    if (!accounts) {
      return
    }

    const moveCallData = encodeMoveCallData(
      "0x95b15595f939a8cf5ece25d44fc832acaf218a6028472b9c0785338fae21bef9::counter::increase",
      [],
      []
    )

    const params = [
      {
        from: accounts[0],
        to: '0xd46e8dd67c5d32be8058bb8eb970870f07244568', //TODO，can be fixed rooch address
        gas: '0x76c0', // 30400
        gasPrice: '0x9184e72a000', // 10000000000000
        value: '0x4e72a', // 2441406250
        data: moveCallData,
      },
    ];

    ethereum
      .request({
        method: 'eth_sendTransaction',
        params,
      })
      .then((result) => {
        // The result varies by RPC method.
        // For example, this method returns a transaction hash hexadecimal string upon success.
        setCounter((v)=>v+1)
      })
      .catch((error) => {
        // If the request fails, the Promise rejects with an error.
      });
  }

  return (
    <div className="App">
      <header className="App-header">
        <div>
          <div>Counter: {counter}</div>
          <button className="Inc-Btn" onClick={handleClick}>Inc</button>
        </div>
      </header>
    </div>
  );
}

export default App;
