// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Buffer } from 'buffer';
import { useState, useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { listFieldStates } from "../utils/index";
import { useRoochClient } from "@roochnetwork/rooch-sdk-kit";
import { BcsType } from "@roochnetwork/rooch-sdk";

export function useRoochFieldStates(
  objectID: string, 
  fieldBcsType: BcsType<any, any>, 
  opts: { refetchInterval: number }
) {
  const client = useRoochClient();
  const [fields, setFields] = useState<Map<string, any>>(new Map<string, any>);
  
  const { data: fieldStats } = useQuery({
    queryKey: ["listFieldStates"],
    queryFn: async () => listFieldStates(client, objectID),
    enabled: !!objectID,
    refetchInterval: opts.refetchInterval,
  });

  const deserializeFieldState = (hexValue: string) => {
    try {
      const cleanHexValue = hexValue.startsWith('0x') ? hexValue.slice(2) : hexValue;
      const buffer = Buffer.from(cleanHexValue, "hex");
      return fieldBcsType.parse(buffer);
    } catch (error) {
      console.error('BCS deserialization error:', error);
      return null;
    }
  };

  useEffect(() => {
    if (fieldStats?.result) {
      const newFields = new Map<string, any>();
      
      for (const item of fieldStats.result) {
        if (item.state?.value) {
          const deserializedValue = deserializeFieldState(item.state.value);
          if (deserializedValue) {
            newFields.set(item.field_key, deserializedValue);
          }
        }
      }
      
      setFields(newFields);
    }
  }, [fieldStats]);

  return { 
    fields: fields,
  };
}
