import {Box, Button, Container, Flex, Heading, Text} from "@radix-ui/themes";
import {
  SupportChain,
  useCurrentSession,
  useWallets,
  useWalletStore,
  useRoochClientQuery, useConnectWallet, useCurrentAccount, useCreateSessionKey
} from "@roochnetwork/rooch-sdk-kit";
import {useState} from "react";

// Your publish counter contract address
const devCounterAddress = ""
const devCounterModule = `${devCounterAddress}::counter`

function App() {
  const account = useCurrentAccount();
  const sessionKey = useCurrentSession();
  const connectionStatus = useWalletStore((state) => state.connectionStatus)
  const wallets = useWallets().filter((wallet) => wallet.getChain() === SupportChain.BITCOIN)
  const [loading, setLoading] = useState(false)
  const [sessionLoading, setSessionLoading] = useState(false)
  const {mutateAsync: connectWallet} = useConnectWallet()
  const {mutateAsync: createSessionKey} = useCreateSessionKey()

  let {data, error, isPending, refetch} = useRoochClientQuery("executeViewFunction", {
    funcId: `${devCounterModule}::value`,
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
        maxInactiveInterval: 1000,
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

  const handlerIncrease = () => {
    if (loading) {
      return
    }

    setLoading(true)

    const func = `${devCounterModule}::increase`

    sessionKey?.sendTransaction(func, [], []).finally(async () => {
      await refetch()
      setLoading(false)
    })
  }

  return (
    <>
      <Flex
        position="sticky"
        px="4"
        py="2"
        justify="between"
        style={{
          borderBottom: "1px solid var(--gray-a2)"
        }}
      >
        <Box>
          <Heading>dApp Counter Template</Heading>
        </Box>

        {wallets.length === 0
          ? "Please install the wallet and try again"
          : connectionStatus !== "disconnected"
            ? connectionStatus
            : (
              <Box>
                <Button
                  onClick={async () => {
                    await connectWallet({
                      wallet: wallets[0],
                    });
                  }}>
                  Connect Wallet
                </Button>
              </Box>
            )
        }
      </Flex>

      <Container
        mt="5"
        pt="2"
        px="4"
        style={{background: "var(--gray-a2)", minHeight: 500}}
      >
        <Box mt="2">
          <Text style={{fontWeight: "bold"}}>Address: </Text>
          <Text style={{wordWrap: "break-word"}}>{account?.address}</Text>
        </Box>

        <Box mt="4">
          <Text style={{fontWeight: "bold"}}>PublicKey: </Text>
          <Text style={{wordWrap: "break-word"}}>{account?.publicKey}</Text>
        </Box>

        <Box mt="4">
          <Text style={{fontWeight: "bold"}}>Compressed PublicKey: </Text>
          <Text style={{wordWrap: "break-word"}}>{account?.compressedPublicKey}</Text>
        </Box>

        <Box mt="4">
          <Text style={{fontWeight: "bold"}}>Session Account Address: </Text>
          <Text style={{wordWrap: "break-word"}}>{sessionKey?.getAddress()}</Text>
        </Box>

        <Heading size="3" mt="6">{sessionKey ? "Counter" : "Create session key"}</Heading>

        {devCounterAddress.length !== 0 ?
          <Flex direction="column" gap="2">
            {sessionKey ? (
              <Text>
                {isPending ? "loading..." : error ? "counter module not published" : `${data?.return_values?.[0]?.decoded_value}`}
              </Text>
            ) : null}
            <Flex direction="row" gap="2" mt="2">
              {
                <Button
                  disabled={loading || sessionLoading}
                  onClick={sessionKey ? handlerIncrease : handlerCreateSessionKey}
                >
                  {sessionKey ? "Increment" : "Create"}
                </Button>
              }
            </Flex>
          </Flex>
          : <><Box>
            <Text>Please refer to the contract published by readme before trying again.</Text>
          </Box>
            <Text>If you have published a contract, enter the contract address correctly into devCounterAddress.</Text>
          </>

        }
      </Container>
    </>
  );
}

export default App;
