// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { config } from "../config/index";
import { useRoochClientQuery } from "@roochnetwork/rooch-sdk-kit";
import { useQuery } from "@tanstack/react-query";
import { useRoochClient } from "@roochnetwork/rooch-sdk-kit";
import { listFieldStates } from "../utils/index"
 
const extractObjectIds = (data: any[]) => {
  const items = data.map(item => {
    if(!item?.state?.decoded_value?.value?.value?.value?.id) {
      console.log("Invalid item structure:", item);
      return null;
    }

    const name = item.state.decoded_value.value.name
    const id = item.state.decoded_value.value.value.value.id;
    return {
      name,
      id,
    };
  }).filter(Boolean); // Remove null values

  return items;
};

export function useGameState() {
  const client = useRoochClient();
  const { data: gameState, refetch: roochFishRefetch } = useRoochClientQuery(
    "getStates",
    {
      accessPath: `/object/${config.gameStateObjectID}`,
      stateOption: {
        decode: true,
      },
    },
    { refetchInterval: 3000 }
  );

  const pondHandleId = (gameState as any)?.[0]?.decoded_value?.value?.ponds?.value?.handle?.value?.id;

  const { data: pondsData } = useQuery({
    queryKey: ["listFieldStates", pondHandleId],
    queryFn: async () => pondHandleId ? listFieldStates(client, pondHandleId) : null,
    enabled: !!pondHandleId,
  });

  const finalGameState = gameState ? {
    ...gameState,
    ponds: extractObjectIds(pondsData ? pondsData.result : [])
  } : null;

  return { data: finalGameState };
}