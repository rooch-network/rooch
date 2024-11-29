// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { config, PondID } from "../config/index";
import { transformObject } from "../utils/rooch_object";
import { bcs } from "@roochnetwork/rooch-sdk";
import { useRoochClient } from "@roochnetwork/rooch-sdk-kit";
import { useRoochState } from "./useRoochStates"
import { useRoochFieldStates } from "./useRoochFieldStates"
import { useRoochWSFieldStates } from "./useRoochWSFieldStates"

const Fish = bcs.struct('Fish', {
  id: bcs.u64(),
  owner: bcs.Address,
  size: bcs.u64(),
  x: bcs.u64(),
  y: bcs.u64(),
});

const Food = bcs.struct('Food', {
  id: bcs.u64(),
  owner: bcs.Address,
  size: bcs.u64(),
  x: bcs.u64(),
  y: bcs.u64(),
});

const FishDynamicField = bcs.struct('DynamicField', {
  name: bcs.u64(),
  value: Fish,
});

const FoodDynamicField = bcs.struct('DynamicField', {
  name: bcs.u64(),
  value: Food,
});

interface FishData {
  id: number;
  owner: string;
  size: number;
  x: number;
  y: number;
}

interface FoodData {
  id: number;
  size: number;
  x: number;
  y: number;
}

interface PondStateData {
  width: number,
  height: number,
  fishes: {
    handle: {
      id: string;
    };
  };
  foods: {
    handle: {
      id: string;
    };
  };

  exit_zones: any,
}

interface DelayRecord {
  timestamp: number;
  delay: number;
}

interface PondStateReturn {
  data: PondStateData | null;
  fishData: FishData[];
  foodData: FoodData[];
  getRecentDelays: any;
}

export function usePondState(pondID: PondID): PondStateReturn {
  const client = useRoochClient();

  const { data, txOrder, refetch: roochFishRefetch } = useRoochState(
    config.ponds[pondID],
    { 
      refetchInterval: 60000,
    }
  );

  const pondData = transformObject(data?data[0]:null)

  const fishTableHandleId = pondData?.fishes?.handle?.id;
  const foodTableHandleId = pondData?.foods?.handle?.id;

  const { fields: fishData, getRecentDelays } = useRoochWSFieldStates(fishTableHandleId, FishDynamicField, {
    refetchInterval: 10000,
    diffInterval: 40,
  });

  const { fields: foodData } = useRoochFieldStates(foodTableHandleId, FoodDynamicField, {
    refetchInterval: 5000,
  });

  const finalPondState = pondData ? {
    ...pondData,
  } : null;

  const finalFishData = transformFish(fishData);
  const finalFoodData = transformFood(foodData);
  
  return { 
    data: finalPondState, 
    fishData: finalFishData, 
    foodData: finalFoodData,
    getRecentDelays
  };
}

function transformFish(fishData: Map<string, any>): Array<any> {
  if (!fishData || !(fishData instanceof Map)) {
    return [];
  }

  return Array.from(fishData.values()).map(field => {
    return {
      id: field.value.id,
      owner: field.value.owner,
      size: field.value.size,
      x: field.value.x,
      y: field.value.y
    };
  });
}

function transformFood(foodData: Map<string, any>): Array<any> {
  if (!foodData || !(foodData instanceof Map)) {
    return [];
  }

  return Array.from(foodData.values()).map(field => {
    return {
      id: field.value.id,
      size: field.value.size,
      x: field.value.x,
      y: field.value.y
    };
  });
}
