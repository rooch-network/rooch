#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

docker run -d --name bitcoind --network host -v $HOME/.bitcoin:/data/.bitcoin lncm/bitcoind:v25.1 -chain=regtest -txindex=1 -fallbackfee=0.00001 -zmqpubrawblock=tcp://0.0.0.0:28332 -zmqpubrawtx=tcp://0.0.0.0:28333 -rpcallowip=0.0.0.0/0 -rpcbind=0.0.0.0 -rpcauth='roochuser:925300af2deda1996d8ff66f2a69dc84$681057d8bdccae2d119411befa9a5f949eff770933fc377816348024d25a2402'
