/* eslint-disable @typescript-eslint/no-explicit-any */
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
// Author: Jason Jo

import { config } from "./config";
import { LoadingButton } from "@mui/lab";
import { Button, Chip, Drawer, Stack, Typography } from "@mui/material";
import { styled } from "@mui/material/styles";
import { Args, Transaction } from "@roochnetwork/rooch-sdk";
import {
  UseSignAndExecuteTransaction,
  useConnectWallet,
  useCreateSessionKey,
  useCurrentAddress,
  useCurrentSession,
  useRemoveSession,
  useRoochClientQuery,
  useWalletStore,
  useWallets,
} from "@roochnetwork/rooch-sdk-kit";
import { enqueueSnackbar } from "notistack";
import { useState } from "react";
import CountUp from "react-countup";
import "./App.css";
import { useRccOwner } from "./hooks/useRccOwner";
import { fNumber, shortAddress } from "./utils";
import { DebugScene } from './scenes/debug_scene'
import { PondScene } from './scenes/pond_scene';
import { useGameState } from './hooks/useGameState';
import { useLatestTransaction } from "./hooks/useLatestTransaction";

function getNextRewardClick(currentClicks: number): number {
  const remainder = currentClicks % 21;
  if (remainder === 0) {
    return currentClicks + 21;
  } else {
    return currentClicks + (21 - remainder);
  }
}

const drawerWidth = 300;

const Main = styled("main", { shouldForwardProp: (prop) => prop !== "open" })<{
  open?: boolean;
}>(({ theme, open }) => ({
  flexGrow: 1,
  alignItems: "center",
  padding: theme.spacing(3),
  transition: theme.transitions.create("margin", {
    easing: theme.transitions.easing.sharp,
    duration: theme.transitions.duration.leavingScreen,
  }),
  marginLeft: `${open ? drawerWidth : "0"}px`,
  ...(open && {
    transition: theme.transitions.create("margin", {
      easing: theme.transitions.easing.easeOut,
      duration: theme.transitions.duration.enteringScreen,
    }),
  }),
}));

// Publish address of the counter contract
const counterAddress =
  "0x872502737008ac71c4c008bb3846a688bfd9fa54c6724089ea51b72f813dc71e";

const roochCounterObject =
  "0x24e093a6fa4698d1b6efd27ae9f1c21057b91bb9a2ef3c0fce2c94b44601764b";

const treasuryObject =
  "0xe7beeda989fa0b2201c945310d533c82027c3270a39e2bcbaa65c4563210db82";

const treasuryOwnerAddress =
  "rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhxqaen";

const rgasCoinType =
  "0x3::gas_coin::RGas";

export const rccCoinStoreType =
  "0x3::coin_store::CoinStore<0x872502737008ac71c4c008bb3846a688bfd9fa54c6724089ea51b72f813dc71e::rooch_clicker_coin::RCC>";

/*
const fetchGameStateInfo = async (client: RoochClient) => {
  const data = await client.getStates({
    accessPath: `/object/${config.gameStateObjectID}`,
    stateOption: {
      decode: true,
    },
  });

  return { result: data };
};
*/
  
function App() {
  const wallets = useWallets();
  const currentAddress = useCurrentAddress();
  const sessionKey = useCurrentSession();
  const connectionStatus = useWalletStore((state) => state.connectionStatus);
  const setWalletDisconnected = useWalletStore(
    (state) => state.setWalletDisconnected
  );
  const { mutateAsync: connectWallet } = useConnectWallet();

  const { mutateAsync: createSessionKey } = useCreateSessionKey();
  const { mutateAsync: removeSessionKey } = useRemoveSession();
  const { mutateAsync: signAndExecuteTransaction } =
    UseSignAndExecuteTransaction();

  const { rccOwnerList } = useRccOwner();

  const [showLeaderboard, setShowLeaderboard] = useState(false);

  console.log("currentAddress:", currentAddress?.genRoochAddress().toStr() || "");
  
  const { data: RCCBalance, refetch: refetchRCCBalance } = useRoochClientQuery(
    "getBalance",
    {
      owner: currentAddress?.genRoochAddress().toStr() || "",
      coinType: rgasCoinType,
    }
  );

  const [sessionLoading, setSessionLoading] = useState(false);
  const [txnLoading, setTxnLoading] = useState(false);
  const handlerCreateSessionKey = async () => {
    if (sessionLoading) {
      return;
    }
    setSessionLoading(true);

    const defaultScopes = [`${config.roochFishAddress}::*::*`];
    createSessionKey(
      {
        appName: "rooch_clicker",
        appUrl: "http://localhost:5173",
        maxInactiveInterval: 3600,
        scopes: defaultScopes,
      },
      {
        onSuccess: (result) => {
          console.log("session key", result);
        },
        onError: (error) => {
          if (String(error).includes("1004")) {
            enqueueSnackbar("Insufficient gas, please claim gas first", {
              variant: "warning",
              action: (
                <a
                  href="https://rooch.network/build/getting-started/get-gas-coin"
                  target="_blank"
                >
                  <Chip
                    size="small"
                    label="Get Rooch Testnet Coin"
                    variant="filled"
                    className="font-semibold !text-slate-50 min-h-10"
                    sx={{
                      background: "#000",
                      borderRadius: "12px",
                      cursor: "pointer",
                    }}
                  />
                </a>
              ),
            });
          } else {
            enqueueSnackbar(String(error), {
              variant: "warning",
            });
          }
        },
      }
    ).finally(() => setSessionLoading(false));
  };

  return (
    <Stack
      className="font-sans min-w-[1024px]"
      direction="column"
      sx={{
        minHeight: "calc(100vh - 4rem)",
      }}
    >
      <Stack justifyContent="space-between" className="w-full">
        <img src="./rooch_black_combine.svg" width="120px" alt="" />
        <Stack spacing={1} justifyItems="flex-end">
          <Chip
            label="Rooch Testnet"
            variant="filled"
            className="font-semibold !bg-slate-950 !text-slate-50 min-h-10"
          />
          <a
            href="https://rooch.network/build/getting-started/get-gas-coin"
            target="_blank"
          >
            <Chip
              label="Get Rooch Testnet Coin"
              variant="filled"
              className="font-semibold !text-slate-50 min-h-10"
              sx={{
                background: "#006BE6",
                borderRadius: "12px",
                cursor: "pointer",
              }}
            />
          </a>
          <Button
            variant="outlined"
            onClick={async () => {
              if (connectionStatus === "connected") {
                setWalletDisconnected();
                return;
              }
              await connectWallet({ wallet: wallets[0] });
            }}
          >
            {connectionStatus === "connected"
              ? shortAddress(currentAddress?.toStr(), 8, 6)
              : "Connect Wallet"}
          </Button>
        </Stack>
      </Stack>
      <Stack className="w-full" justifyContent="space-between">
        <Stack>
          <Typography className="text-4xl font-semibold mt-6 text-left w-full mb-4">
            Rooch Fish |{" "}
            {RCCBalance && (
              <span className="text-2xl">
                Balance: {fNumber(RCCBalance.balance.toString(), 8)} RGas{" "}
                <span className="text-xs ml-2">( Rooch Gas Coin )</span>
              </span>
            )}
          </Typography>
        </Stack>{" "}
        <Stack className="w-1/3" justifyContent="flex-end">
          {!sessionKey ? (
            <LoadingButton
              loading={sessionLoading}
              variant="contained"
              className="!mt-4"
              disabled={connectionStatus !== "connected"}
              onClick={() => {
                handlerCreateSessionKey();
              }}
            >
              {connectionStatus !== "connected"
                ? "Please connect wallet first"
                : "Create Session Key"}
            </LoadingButton>
          ) : (
            <Button
              variant="contained"
              className="!mt-4"
              onClick={() => {
                removeSessionKey({ authKey: sessionKey.getAuthKey() });
              }}
            >
              Clear Session Key
            </Button>
          )}
        </Stack>
      </Stack>
      <Stack
        className="mt-4 w-full font-medium "
        direction="column"
        alignItems="center"
      >
        <Drawer
          sx={{
            width: drawerWidth,
            flexShrink: 0,
            "& .MuiDrawer-paper": {
              width: drawerWidth,
              boxSizing: "border-box",
              marginTop: "168px",
              height: "calc(100% - 168px)",
              background: "transparent",
              p: 2,
            },
          }}
          variant="persistent"
          anchor="left"
          open={showLeaderboard}
        >
          <Stack>
            <Typography className="text-xl font-semibold">
              Leaderboard
            </Typography>
          </Stack>
          <Stack direction="column" className="mt-4" spacing={1.5}>
            {rccOwnerList
              ?.filter((i) => i.owner !== treasuryOwnerAddress)
              .sort((a, b) => {
                return (
                  Number((b.decoded_value?.value.balance as any).value.value) -
                  Number((a.decoded_value?.value.balance as any).value.value)
                );
              })
              .map((item: any, i: number) => {
                return (
                  <Stack
                    key={"stack-" + i}
                    className="w-full"
                    justifyContent="space-between"
                    sx={{
                      fontWeight:
                      item.owner === currentAddress?.genRoochAddress().toStr()
                          ? 700
                          : 500,
                    }}
                  >
                    <Typography>
                      {shortAddress(item.owner_bitcoin_address, 6, 6)}
                    </Typography>
                    <Typography
                      style={{
                        fontVariantNumeric: "tabular-nums lining-nums",
                      }}
                    >
                      {fNumber(
                        Number(
                          (item.decoded_value?.value.balance as any).value.value
                        ),8
                      )}
                    </Typography>
                  </Stack>
                );
              })}
          </Stack>
        </Drawer>
        <Main open={showLeaderboard}>
          {config.debug ? (
            <DebugScene />
          ):(
            <PondScene />
          )}
        </Main>
      </Stack>
    </Stack>
  );
}

export default App;
