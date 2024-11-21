// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Buffer } from 'buffer';
import { useState, useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { getLatestTransaction } from "../utils/rooch_client";
import { listFieldStates, syncStates } from "../utils/index";
import { useRoochWSClient } from "./useRoochWSClient";
import { BcsType } from "@roochnetwork/rooch-sdk";
import { useTransactionDelay } from './useTransactionDelay';

interface RoochWSFieldStatesResult<T> {
  fields: Map<string, T>;
  stateRoot: string | undefined;
  isLoading: boolean;
  startTracking: (txOrder: string) => void;
  recordTxConfirm: (tempId: string, txOrder: string) => void;
  recordStateSync: (txOrder: string) => void;
  getRecentDelays: () => any;
}

export function useRoochWSFieldStates<T>(
  objectID: string, 
  fieldBcsType: BcsType<any, any>, 
  opts: { 
    refetchInterval: number,
    diffInterval: number
  }
): RoochWSFieldStatesResult<T> {
  const client = useRoochWSClient();
  const [fields, setFields] = useState<Map<string, any>>(new Map<string, any>);
  const previousStateRootRef = useRef<string | undefined>();
  const lastValidFieldsRef = useRef<Map<string, any>>(new Map<string, any>());
  const isFetchingRef = useRef(false);
  const processedTxOrdersRef = useRef<Set<string | undefined>>(new Set());

  const { 
    startTracking, 
    recordTxConfirm, 
    recordStateSync, 
    getRecentDelays 
  } = useTransactionDelay();

  const { data: latestTxData } = useQuery({
    queryKey: ["rooch_latest_tx_for_use_rooch_ws_field_states"],
    queryFn: async () => getLatestTransaction(client),
    enabled: !!objectID,
    refetchInterval: opts.refetchInterval,
    staleTime: opts.refetchInterval,
  });

  const stateRoot = latestTxData?.execution_info?.state_root;
  const txOrder = latestTxData?.transaction?.sequence_info.tx_order;
  const cursorRef = useRef(txOrder);

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

  const updateFields = (newFields: Map<string, any>, isFullData: boolean = false) => {
    if (isFullData) {
      // For full data fetch, always update regardless of size
      lastValidFieldsRef.current = newFields;
      setFields(newFields);
    } else if (newFields.size > 0) {
      // For diff updates, only update if there are changes
      lastValidFieldsRef.current = newFields;
      setFields(newFields);
    } else {
      setFields(lastValidFieldsRef.current);
    }
  };

  // Add stateRoot validation to prevent unnecessary resets
  useEffect(() => {
    if (!stateRoot) return;
    
    // Only trigger full data fetch if stateRoot changed
    if (previousStateRootRef.current && previousStateRootRef.current !== stateRoot) {
      isFetchingRef.current = true;
      fetchFullData().finally(() => {
        isFetchingRef.current = false;
      });
    }
    previousStateRootRef.current = stateRoot;
  }, [stateRoot]);

  const fetchFullData = async () => {
    if (!objectID || !stateRoot || !client) return;
    
    try {
      const fieldStats = await listFieldStates(client, objectID, stateRoot);
      
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
        
        updateFields(newFields, true);
      }
    } catch (error) {
      console.error("Error fetching field states:", error);
    }
  };

  const fetchDiffData = async () => {
    if (!objectID || !cursorRef.current || !client || isFetchingRef.current) return;

    try {
      const fieldStats = await syncStates(client, objectID, cursorRef.current);
      
      for (const changeSet of fieldStats.result) {
        if (BigInt(changeSet.tx_order) > BigInt(cursorRef.current)) {
          cursorRef.current = (BigInt(changeSet.tx_order) + BigInt(1)).toString();
        }

        if (changeSet.state_change_set.changes.length === 0) continue;

        const newFields = new Map(lastValidFieldsRef.current);
        
        for (const change of changeSet.state_change_set.changes) {
          for (const field of change.fields) {
            const fieldKey = "0x" + field.metadata.id.slice(-64);
            
            if (field.value?.new || field.value?.modify) {
              const value = field.value.new || field.value.modify;
              const deserializedValue = deserializeFieldState(value);
              if (deserializedValue) {
                newFields.set(fieldKey, deserializedValue);
              }
            } else if (field.value?.delete) {
              newFields.delete(fieldKey);
            }
          }
        }
        
        updateFields(newFields);

        // Record state sync timing
        if (txOrder && !processedTxOrdersRef.current.has(txOrder)) {
          recordStateSync(txOrder);
          processedTxOrdersRef.current.add(txOrder);
          
          // Clean up old processed tx orders (keep last 100)
          if (processedTxOrdersRef.current.size > 100) {
            const orders = Array.from(processedTxOrdersRef.current);
            orders.slice(0, orders.length - 100).forEach(order => {
              processedTxOrdersRef.current.delete(order);
            });
          }
        }
      }
    } catch (error) {
      console.error("Error fetching diff states:", error);
    }
  };

  useEffect(() => {
    if (!objectID || !stateRoot || !txOrder) {
      return;
    }

    cursorRef.current = txOrder;
    
    // Initial fetch
    if (!isFetchingRef.current) {
      fetchFullData();
    }

    const intervalId = setInterval(fetchDiffData, opts.diffInterval);

    return () => {
      clearInterval(intervalId);
    };
  }, [stateRoot, txOrder, objectID, fieldBcsType, client, opts.diffInterval]);

  return {
    fields: fields.size > 0 ? fields : lastValidFieldsRef.current,
    stateRoot,
    isLoading: isFetchingRef.current,

    // Export delay tracking functions
    startTracking,
    recordTxConfirm,
    recordStateSync,
    getRecentDelays,
  };
}
