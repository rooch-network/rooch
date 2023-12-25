// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React from "react";
import ReactDOM from "react-dom/client";
import "./index.css";
import "@radix-ui/themes/styles.css";

import { WalletProvider, RoochClientProvider } from "@roochnetwork/rooch-sdk-kit"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Theme } from "@radix-ui/themes";
import { LocalChain } from "@roochnetwork/rooch-sdk";

// import App from "./App";
import TestBitcoin from "./TestBitcoin.tsx";

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Theme appearance="dark">
      <QueryClientProvider client={queryClient}>
            <RoochClientProvider defaultChain={LocalChain}>
                <WalletProvider autoConnect={false}>
                  <TestBitcoin/>
                </WalletProvider>
            </RoochClientProvider>
      </QueryClientProvider>
    </Theme>
  </React.StrictMode>
);
