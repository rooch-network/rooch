import { useMemo } from 'react'
import { DEFAULT_CREATE_WS_CLIENT } from '../clients/rooch/wsClient'
import { useRoochContext } from "@roochnetwork/rooch-sdk-kit";

export function useRoochWSClient() {
  const { network: currentNetwork, networks } = useRoochContext()

  const client = useMemo(() => {
    console.log("create rooch ws client")
    return DEFAULT_CREATE_WS_CLIENT(currentNetwork, networks[currentNetwork])
  }, [currentNetwork, networks])

  return client
}
