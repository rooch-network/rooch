import logo from './logo.svg';
import './App.css';
import { useEffect, useState } from "react";
import MetaMaskSDK from '@metamask/sdk';
import { arrayify, hexlify } from '@ethersproject/bytes';
import { toUtf8Bytes } from '@ethersproject/strings';
import { keccak256 } from '@ethersproject/keccak256';

const options = {
  dappMetadata:{},
};
const MMSDK = new MetaMaskSDK(options);
const ethereum = MMSDK.getProvider(); // You can also access via window.ethereum

function App() {
  let [counter, setCounter] = useState<number>(1)

  const encodeMoveCall = (functionId:string, tyArgs: string[], args:string[])=>{
    const encodedTypeArgs = utils.tx.encodeStructTypeTags(tyArgs)
    const scriptFunction = utils.tx.encodeScriptFunction(functionId, encodedTypeArgs, args)

    // Multiple BcsSerializers should be used in different closures, otherwise, the latter will be contaminated by the former.
    const payloadInHex = (function () {
      const se = new bcs.BcsSerializer()
      scriptFunction.serialize(se)
      return hexlify(se.getBytes())
    })()

    return payloadInHex;
  }

  const wrapAsEthCallData = (moveAction)=>{
    // 创建方法签名
    const signature = 'moveAction(bytes)';

    // 计算函数选择器
    const keccak256Hash = keccak256(toUtf8Bytes(signature));
    const functionSelector = keccak256Hash.slice(0, 10); // 截取前4字节

    // 拼接函数选择器与编码参数
    return functionSelector + moveAction; // 删除'0x'前缀
  }

  const handleClick = async ()=>{
    let accounts = await ethereum.request({ method: 'eth_requestAccounts', params: [] });

    let moveCallData = encodeMoveCall(
      "0x95b15595f939a8cf5ece25d44fc832acaf218a6028472b9c0785338fae21bef9::counter::increase",
      [],
      []
    )

    let ethCallData = wrapAsEthCallData(moveCallData)

    let params = [
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
