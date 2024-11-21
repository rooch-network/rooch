import { config } from "../config/index";
import { useRoochClient, useCurrentAddress } from "@roochnetwork/rooch-sdk-kit";
import { RoochClient, Args } from "@roochnetwork/rooch-sdk";
import { useQuery } from "@tanstack/react-query";

const queryUserFishs = async (client: RoochClient, pondID: number, playerAddress: any) => {
  const data = await client.executeViewFunction({
    target: `${config.roochFishAddress}::rooch_fish::get_pond_player_fish_ids`,
    args: [
      Args.objectId(config.gameStateObjectID),
      Args.u64(BigInt(pondID)),
      Args.address(playerAddress.roochAddress)
    ],
  });

  return { data };
};

export function usePlayerState(pondID: number) {
  const currentAddress = useCurrentAddress();
  const client = useRoochClient();

  const { data } = useQuery({
    queryKey: ["query_user_fishs"],
    queryFn: async () => queryUserFishs(client, pondID, currentAddress),
    refetchInterval: 5000 
  });

  const fish_ids = (data?.data?.return_values?.[0]?.decoded_value ?? null) as Array<number>;
  return { fish_ids };
}
