// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useRef, useCallback } from 'react';

interface DelayRecord {
  txOrder: string;
  sendTime: number;
  confirmTime: number;
  syncTime: number;
  totalDelay: number;
  syncDelay: number;
}

interface PendingTx {
  tempId: string;
  sendTime: number;
  txOrder?: string;
  confirmTime?: number;
}

// Move these to module level
const delayRecords: DelayRecord[] = [];
const pendingTxs: Map<string, PendingTx> = new Map();
const txOrderMap: Map<string, string> = new Map();

export function useTransactionDelay() {
  const startTracking = useCallback(() => {
    const tempId = crypto.randomUUID();
    pendingTxs.set(tempId, {
      tempId,
      sendTime: Date.now(),
    });
    return tempId;
  }, []);

  const recordTxConfirm = useCallback((tempId: string, txOrder: string) => {
    const pending = pendingTxs.get(tempId);
    if (pending) {
      pending.txOrder = txOrder;
      pending.confirmTime = Date.now();
      txOrderMap.set(txOrder, tempId);
    }
  }, []);

  const recordStateSync = useCallback((txOrder: string) => {
    const tempId = txOrderMap.get(txOrder);
    if (!tempId) return;

    const pending = pendingTxs.get(tempId);
    if (!pending || !pending.confirmTime) return;

    const now = Date.now();
    const record: DelayRecord = {
      txOrder,
      sendTime: pending.sendTime,
      confirmTime: pending.confirmTime,
      syncTime: now,
      totalDelay: now - pending.sendTime,
      syncDelay: now - pending.confirmTime,
    };

    delayRecords.unshift(record);
    if (delayRecords.length > 5) {
      delayRecords.pop();
    }

    pendingTxs.delete(tempId);
    txOrderMap.delete(txOrder);
  }, []);

  const getRecentDelays = useCallback(() => {
    return delayRecords;
  }, []);

  return {
    startTracking,
    recordTxConfirm,
    recordStateSync,
    getRecentDelays,
  };
}