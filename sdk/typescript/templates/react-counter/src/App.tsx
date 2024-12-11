import {Box, Button, Container, Flex, Heading, Text} from "@radix-ui/themes";
import {
  useCurrentSession,
  useWallets,
  useRoochClientQuery, useConnectWallet, useCreateSessionKey,
  useRoochClient,
  useCurrentWallet
} from "@roochnetwork/rooch-sdk-kit";

import {useState} from "react";
import { Transaction } from '@roochnetwork/rooch-sdk'

// Your publish counter contract address
const devCounterAddress = ""
const devCounterModule = `${devCounterAddress}::counter`

function App() {
  const sessionKey = useCurrentSession();
  const currentWallet = useCurrentWallet();
  const client = useRoochClient()
  const wallets = useWallets()
  const [loading, setLoading] = useState(false)
  const [sessionLoading, setSessionLoading] = useState(false)
  const {mutateAsync: connectWallet} = useConnectWallet()
  const {mutateAsync: createSessionKey} = useCreateSessionKey()

  const {isConnected, status, wallet} = currentWallet

  let {data, error, isPending, refetch} = useRoochClientQuery("executeViewFunction", {
    target: `${devCounterModule}::value`,
  })

  const handlerCreateSessionKey = () => {
    if (sessionLoading) {
      return
    }
    setSessionLoading(true)

    const defaultScopes = [
      `${devCounterAddress}::*::*`,
    ]
    createSessionKey(
      {
        appName: "rooch_test",
        appUrl: "https://test.com",
        scopes: defaultScopes
      },
      {
        onSuccess: (result) => {
          console.log("session key", result);
        },
        onError: (why) => {
          console.log(why)
        }
      },
    ).finally(() => setSessionLoading(false))
  }

  const handlerIncrease = async () => {
    if (loading) {
      return
    }

    setLoading(true)

    const tx = new Transaction()
    tx.callFunction({
      target: `${devCounterModule}::increase`
    })

    const result = await client.signAndExecuteTransaction({
      transaction: tx,
      signer: sessionKey!
    })

    if (result.execution_info.status.type !== 'executed') {
      console.log('increase failed')
    }

    refetch()
    setLoading(false)
  }


  return (
    <>
      <Flex
        position="sticky"
        px="4"
        py="2"
        justify="between"
        style={{
          borderBottom: "1px solid var(--gray-a2)",
        }}
      >
        <Box>
          <Heading>dApp Counter Template</Heading>
        </Box>
        {wallets.length === 0 ? (
          "Please install the wallet and try again"
        ) : isConnected ? (
          status
        ) : (
          <Box>
            <Button
              onClick={async () => {
                try {
                  await connectWallet({
                    wallet: wallets[0],
                  });
                } catch (e) {
                  console.log(e)
                }
              }}
            >
              Connect Wallet
            </Button>
          </Box>
        )}
      </Flex>

      <Container
        mt="5"
        pt="2"
        px="4"
        style={{ background: "var(--gray-a2)", minHeight: 500 }}
      >
        <Box mt="2">
          <Text style={{ fontWeight: "bold" }}>Address: </Text>
          <Text style={{ wordWrap: "break-word" }}>
            {wallet?.getBitcoinAddress().toStr()}
          </Text>
        </Box>

        <Box mt="4">
          <Text style={{ fontWeight: "bold" }}>PublicKey: </Text>
          <Text style={{ wordWrap: "break-word" }}>
            {wallet?.getPublicKey().toString()}
          </Text>
        </Box>

        <Box mt="4">
          <Text style={{ fontWeight: "bold" }}>Session Address: </Text>
          <Text style={{ wordWrap: "break-word" }}>
            {sessionKey?.getRoochAddress()?.toStr()}
          </Text>
        </Box>

        <Heading size="3" mt="6">
          {sessionKey ? "Counter" : "Create session key"}
        </Heading>

        <Button onClick={async () => {
          if (wallet) {
            const b = await wallet.getBalance()
            console.log(b)
          }

          wallet?.sendBtc({ // pr tb1qxvrzdqlnmpzxr6zsg7g2c62gu6l33qxzz6z5l2
            toAddress: 'tb1qxvrzdqlnmpzxr6zsg7g2c62gu6l33qxzz6z5l2',
            satoshis: 10000000
          })
        }
        }>
          trn
        </Button>

        {devCounterAddress.length !== 0 ? (
          <Flex direction="column" gap="2">
            {sessionKey ? (
              <Text>
                {isPending
                  ? "loading..."
                  : error
                    ? "counter module not published"
                    : `${data?.return_values?.[0]?.decoded_value}`}
              </Text>
            ) : null}
            <Flex direction="row" gap="2" mt="2">
              {
                <Button
                  disabled={loading || sessionLoading}
                  onClick={
                    sessionKey ? handlerIncrease : handlerCreateSessionKey
                  }
                >
                  {sessionKey ? "Increment" : "Create"}
                </Button>
              }
            </Flex>
          </Flex>
        ) : (
          <>
            <Box>
              <Text>
                Please refer to the contract published by readme before trying
                again.
              </Text>
            </Box>
            <Text>
              If you have published a contract, enter the contract address
              correctly into devCounterAddress.
            </Text>
          </>
        )}
      </Container>
    </>
  );
}

export default App;
