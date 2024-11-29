// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Basic types for coordinates and dimensions
export type Point = {
  x: number;
  y: number;
};

// Fish type definition
export type Fish = {
  id: number;
  owner: string; // address as string
  size: number;
  x: number;
  y: number;
};

// Food type definition
export type Food = {
  id: number;
  size: number;
  x: number;
  y: number;
};

// Player related types
export type PlayerState = {
  owner: string; // address as string
  feed_amount: string; // u256 as string
  reward: string; // u256 as string
  fish_count: number;
  fish_ids: number[];
};

export type PlayerList = {
  players: Record<string, PlayerState>; // address -> PlayerState mapping
  total_feed: string; // u256 as string
  player_count: number;
};

// ExitZone type definition
export type ExitZone = {
  x: number;
  y: number;
  radius: number;
};

// Treasury type definition
export type Treasury = {
  coin_store: {
    value: string; // Using string for large numbers/addresses
  };
};

// Main PondState type definition
export type PondState = {
  id: number;
  fishes: Record<number, Fish>;
  foods: Record<number, Food>;
  exit_zones: ExitZone[];
  quad_tree: any; // Define specific quad tree type if needed
  fish_count: number;
  food_count: number;
  width: number;
  height: number;
  purchase_amount: string; // Using string for u256
  next_fish_id: number;
  next_food_id: number;
  max_fish_count: number;
  max_food_count: number;
  treasury: Treasury;
  player_list: PlayerList;
};
