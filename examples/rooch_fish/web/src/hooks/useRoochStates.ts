// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useRoochClientQuery } from "@roochnetwork/rooch-sdk-kit";
import { useQuery } from "@tanstack/react-query";
import { useRoochClient } from "@roochnetwork/rooch-sdk-kit";
import { getLatestTransaction } from "../utils/rooch_client"

export function useRoochState(objectID: string, opts: any) {
  const client = useRoochClient();

  const { data: latestTxData } = useQuery({
    queryKey: ["rooch_latest_tx_for_use_rooch_states"],
    queryFn: async () => getLatestTransaction(client),
    enabled: !!objectID,
    refetchInterval: opts.refetchInterval,
  });

  //console.log("useRoochState latestTxData:", latestTxData)

  const stateRoot = latestTxData?.execution_info?.state_root;
  const txOrder = latestTxData?.transaction?.sequence_info.tx_order;

  const { data, refetch: refetch } = useRoochClientQuery(
    "getStates",
    {
      accessPath: `/object/${objectID}`,
      stateOption: {
        decode: true,
        stateRoot: stateRoot
      } as any,
    },
    { 
      enabled: !!stateRoot,
      refetchInterval: opts.refetchInterval,
    }
  );

  return { 
    data: data,
    stateRoot: stateRoot,
    txOrder: txOrder, 
    refetch: refetch,
  };
}
