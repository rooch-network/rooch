// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import "./App.css";
import {Box, Container, Flex, Heading} from "@radix-ui/themes";
import {useWalletStore, useCurrentWallet} from '@roochnetwork/rooch-sdk-kit';

function App() {

    const accounts = useWalletStore((state) => state.accounts)
    const wallet = useCurrentWallet()

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
                    <Heading>dApp Starter Template</Heading>
                </Box>

            </Flex>
            <Container>
                <Container
                    mt="5"
                    pt="2"
                    px="4"
                    style={{background: "var(--gray-a2)", minHeight: 500}}
                >

                    Wallet state: {wallet.status}
                    <p>Address: {accounts.map(v => v.getAddress())}</p>
                </Container>
            </Container>
        </>
    );
}

export default App;

