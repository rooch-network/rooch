/**
 * Copyright (c) Facebook, Inc. and its affiliates
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

export type Optional<T> = T | null;
export type Seq<T> = T[];
export type Tuple<T extends any[]> = T;
export type ListTuple<T extends any[]> = Tuple<T>[];

export type unit = null;
export type bool = boolean;
export type int8 = number;
export type int16 = number;
export type int32 = number;
export type int64 = bigint;
export type int128 = bigint;
export type uint8 = number;
export type uint16 = number;
export type uint32 = number;
export type uint64 = bigint;
export type uint128 = bigint;
export type float32 = number;
export type float64 = number;
export type char = string;
export type str = string;
export type bytes = Uint8Array;
