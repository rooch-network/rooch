/**
 * Copyright (c) Facebook, Inc. and its affiliates
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

export interface Serializer {
  serializeStr(value: string): void;

  serializeBytes(value: Uint8Array): void;

  serializeBool(value: boolean): void;

  serializeUnit(value: null): void;

  serializeChar(value: string): void;

  serializeF32(value: number): void;

  serializeF64(value: number): void;

  serializeU8(value: number): void;

  serializeU16(value: number): void;

  serializeU32(value: number): void;

  serializeU64(value: bigint | number): void;

  serializeU128(value: bigint | number): void;

  serializeI8(value: number): void;

  serializeI16(value: number): void;

  serializeI32(value: number): void;

  serializeI64(value: bigint | number): void;

  serializeI128(value: bigint | number): void;

  serializeLen(value: number): void;

  serializeVariantIndex(value: number): void;

  serializeOptionTag(value: boolean): void;

  getBufferOffset(): number;

  getBytes(): Uint8Array;

  sortMapEntries(offsets: number[]): void;
}
