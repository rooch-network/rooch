// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { test, expect } from "vitest"
import { RoochAccount } from "../../account/rooch-account"
import { HTTPTransport, RequestManager } from "@open-rpc/client-js"
import { Transport } from "@open-rpc/client-js/build/transports/Transport"
import { JsonRpcProvider } from "../../provider/json-rpc-provider"
// import { RoochClient } from "../../generated/client"

test("account", async () => {

  // const transport = new HTTPTransport("http://127.0.0.1:50051", {
  //   headers: {
  //     'Content-Type': 'application/json',
  //     'Client-Sdk-Type': 'typescript',
  //     'Client-Sdk-Version': '1',
  //     'Client-Target-Api-Version': '1',
  //   },
  // });

  // let rq = new RequestManager([transport])
  // let rc = new RoochClient(rq)

  // let s = await rc.rooch_getTransactionByHash("0x98f7083b4a26826d32a33c719f8d71f8d6539ca6cf065d713fb90c0ec9ca02a3")

  // console.log(s)
  // 

  let provider = new JsonRpcProvider()

  let s = await provider.rooch_getTransactionByHash("0x98f7083b4a26826d32a33c719f8d71f8d6539ca6cf065d713fb90c0ec9ca02a3")
  
  console.log(s)

  // old dog slight degree must adult owner pelican canvas best wage erode
  // 0xbcde0bfa380000945caa8b611e6122802e33c9eb7e42f8a9a0bfb4cb6fa7853a
  const m = "old dog slight degree must adult owner pelican canvas best wage erode"

  // m/44'/784'/0'/0'/0'
  const account = RoochAccount.fromDerivePath("m/44'/784'/0'/0'/0'", m)

  console.log(account.address())

  expect(RoochAccount.isValidPath(m))
})
