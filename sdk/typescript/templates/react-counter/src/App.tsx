import { Box, Button, Container, Flex, Heading, Text } from "@radix-ui/themes";
import {
  useCurrentSession,
  useRoochClientQuery,
  useRoochClient,
  ConnectButton,
  SessionKeyGuard,
} from "@roochnetwork/rooch-sdk-kit";

import { useState } from "react";
import { Transaction } from "@roochnetwork/rooch-sdk";
import { useNetworkVariable } from "./networks.ts";

function App() {
  const sessionKey = useCurrentSession();
  const client = useRoochClient();
  const [loading, setLoading] = useState(false);
  const devCounterAddress = useNetworkVariable("counterPackageId");
  const devCounterModule = `${devCounterAddress}::counter`;
  let { data, error, isPending, refetch } = useRoochClientQuery(
    "executeViewFunction",
    {
      target: `${devCounterModule}::value`,
    },
  );

  const handlerIncrease = async () => {
    if (loading) {
      return;
    }

    setLoading(true);

    const tx = new Transaction();
    tx.callFunction({
      target: `${devCounterModule}::increase`,
    });

    const result = await client.signAndExecuteTransaction({
      transaction: tx,
      signer: sessionKey!,
    });

    if (result.execution_info.status.type !== "executed") {
      console.log("increase failed");
    }

    refetch();
    setLoading(false);
  };

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
        <ConnectButton />
      </Flex>

      <Container
        mt="5"
        pt="2"
        px="4"
        style={{ width: "100%", background: "var(--gray-a2)", minHeight: 500 }}
      >
        <Flex
          style={{ flexDirection: "column", alignItems: "center", gap: 10 }}
        >
          <Text style={{ fontSize: 100 }}>
            {data?.return_values
              ? (data.return_values[0].decoded_value as string)
              : 0}
          </Text>
          <SessionKeyGuard onClick={handlerIncrease}>
            <Button disabled={loading || isPending}>Increment</Button>
          </SessionKeyGuard>
          {error && (
            <>
              <Text>
                Please refer to the contract published by readme before trying
                again.
              </Text>
              <Text>
                If you have published a contract, enter the contract address
                correctly into devCounterAddress.
              </Text>
            </>
          )}
        </Flex>
      </Container>
    </>
  );
}

export default App;
