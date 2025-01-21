
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { 
  RoochClientOptions, 
  RoochClient, 
  isRoochClient
} from "@roochnetwork/rooch-sdk";
import { RoochWebSocketTransport} from "./wsTransport"

export const DEFAULT_CREATE_WS_CLIENT = (
  _name: string,
  config: RoochClientOptions | RoochClient,
) => {
  if (isRoochClient(config)) {
    return config
  }

  config.transport = new RoochWebSocketTransport(
    {
      url: config.url!.toString(),
    },
  )

  return new RoochClient(config)
}