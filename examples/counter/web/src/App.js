import logo from './logo.svg';
import './App.css';
import { useEffect, useState } from "react";
import MetaMaskSDK from '@metamask/sdk';
import { arrayify, hexlify } from '@ethersproject/bytes';
import { providers, utils, bcs, encoding, version as starcoinVersion } from '@starcoin/starcoin';

const options = {};
const MMSDK = new MetaMaskSDK(options);
const ethereum = MMSDK.getProvider(); // You can also access via window.ethereum

function App() {
  let [counter, setCounter] = useState(1)

  const encodeMoveCall = async (functionId, strTypeArgs, args)=>{
    const tyArgs = utils.tx.encodeStructTypeTags(strTypeArgs)
    const scriptFunction = utils.tx.encodeScriptFunction(functionId, tyArgs, args)
    console.log(scriptFunction)

    // Multiple BcsSerializers should be used in different closures, otherwise, the latter will be contaminated by the former.
    const payloadInHex = (function () {
      const se = new bcs.BcsSerializer()
      scriptFunction.serialize(se)
      return hexlify(se.getBytes())
    })()

    return payloadInHex;
  }

  const handleClick = async ()=>{
    let accounts = await ethereum.request({ method: 'eth_requestAccounts', params: [] });
    console.log("accounts:", accounts[0]);

    let callData = encodeMoveCall(
      "0xd46e8dd67c5d32be8058bb8eb970870f07244567::counter::increase",
      [],
      []
    )

    console.log("callData:", callData);
    
    let params = [
      {
        from: accounts[0],
        to: '0xd46e8dd67c5d32be8058bb8eb970870f07244567',
        gas: '0x76c0', // 30400
        gasPrice: '0x9184e72a000', // 10000000000000
        value: '0x4e72a', // 2441406250
        data: callData,
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
