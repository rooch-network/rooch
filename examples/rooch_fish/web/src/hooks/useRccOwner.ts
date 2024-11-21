// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useRoochClient } from "@roochnetwork/rooch-sdk-kit";
import { rccCoinStoreType } from "../App";
import { IndexerObjectStateView, RoochClient } from "@roochnetwork/rooch-sdk";
import { useQuery } from "@tanstack/react-query";

const fetchAllOwner = async (client: RoochClient) => {
  let result: IndexerObjectStateView[] = [];
  let cursor = null;
  // eslint-disable-next-line no-constant-condition
  while (true) {
    const data = await client.queryObjectStates({
      filter: {
        object_type: rccCoinStoreType,
      },
      queryOption: {
        decode: true,
        descending: false,
      },
      cursor,
    });

    cursor = data.next_cursor ?? null;
    result = result.concat(data.data);
    if (!data.has_next_page) {
      break;
    }
  }
  return { result };
};

export function useRccOwner() {
  const client = useRoochClient();
  const { data } = useQuery({
    queryKey: ["rcc_owner_list"],
    queryFn: async () => fetchAllOwner(client),
  });
  return { rccOwnerList: data?.result };
}
