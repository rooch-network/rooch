import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useRef } from "react";
import { useRoochWSClient } from "./useRoochWSClient";

export function useSeqNumber(address: string) {
  const client = useRoochWSClient();
  const queryClient = useQueryClient();
  const timerRef = useRef<any>();
  
  const queryKey = ["rooch_seq_number", address];
  
  const { data: seqNumber } = useQuery({
    queryKey,
    queryFn: async () => {
      return client.getSequenceNumber(address);
    },
    refetchOnWindowFocus: false,
  });

  const syncSeqNumber = useCallback(async () => {
    const chainSeq = await client.getSequenceNumber(address);
    console.log("syncSeqNumber:", chainSeq)
    queryClient.setQueryData(queryKey, chainSeq);
  }, [address, client, queryClient, queryKey]);

  const incrementLocal = useCallback(() => {
    

    queryClient.setQueryData(queryKey, (old: bigint | undefined) => {
      if (old === undefined) return 0n;

      console.log("incrementLocal:", old + 1n)
      return old + 1n;
    });

    if (timerRef.current) {
      clearInterval(timerRef.current);
    }
    
    timerRef.current = setInterval(syncSeqNumber, 5000);
  }, [queryClient, queryKey, syncSeqNumber]);

  useEffect(() => {
    return () => {
      if (timerRef.current) {
        clearInterval(timerRef.current);
      }
    };
  }, []);

  return {
    seqNumber: seqNumber ?? 0n,
    incrementLocal,
  };
}
