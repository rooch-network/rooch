import { useRoochClient } from "@roochnetwork/rooch-sdk-kit";
import { useQuery } from "@tanstack/react-query";
import { getTransactionsByOrder } from "../utils/rooch_client"

export function useLatestTransaction() {
  const client = useRoochClient();
  const { data } = useQuery({
    queryKey: ["rooch_latest_tx"],
    queryFn: async () => getTransactionsByOrder(client, null, 1, true),
  });
  
  return { tx: data?.result };
}
