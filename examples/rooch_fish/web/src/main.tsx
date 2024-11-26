// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
// Author: Jason Jo

import "@fontsource-variable/plus-jakarta-sans";
import "@fontsource-variable/raleway";
import { ThemeProvider, styled } from "@mui/material";
import { RoochProvider, WalletProvider } from "@roochnetwork/rooch-sdk-kit";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MaterialDesignContent, SnackbarProvider } from "notistack";
import ReactDOM from "react-dom/client";
import TagManager from "react-gtm-module";
import App from "./App.tsx";
import "./index.css";

import { config } from "./config/index"
import { networkConfig } from "./networks.ts";
import { theme } from "./theme.ts";

const tagManagerArgs = {
  gtmId: "G-8NGQS317V8",
};

TagManager.initialize(tagManagerArgs);

const queryClient = new QueryClient();

// eslint-disable-next-line react-refresh/only-export-components
const StyledContent = styled(MaterialDesignContent)(() => ({
  "&.notistack-MuiContent": {
    borderRadius: "12px",
  },
  "&.notistack-MuiContent-success": {
    borderRadius: "12px",
  },
  "&.notistack-MuiContent-error": {
    borderRadius: "12px",
  },
}));

ReactDOM.createRoot(document.getElementById("root")!).render(
  <ThemeProvider theme={theme}>
    <QueryClientProvider client={queryClient}>
      <RoochProvider networks={networkConfig} defaultNetwork={config.network}>
        <WalletProvider chain={"bitcoin"} autoConnect>
          <SnackbarProvider
            Components={{
              success: StyledContent,
              error: StyledContent,
              warning: StyledContent,
              info: StyledContent,
            }}
          />
          <App />
        </WalletProvider>
      </RoochProvider>
    </QueryClientProvider>
  </ThemeProvider>
);
