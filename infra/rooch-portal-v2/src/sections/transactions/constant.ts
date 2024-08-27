import type { ReactNode } from 'react';
import type { ChipProps } from '@mui/material';

import { TransactionType, TransactionStatus, TransactionAction } from './types';

export const TRANSACTION_STATUS_TYPE_MAP: Record<
  TransactionStatus,
  {
    text: ReactNode;
    color: ChipProps['color'];
  }
> = {
  [TransactionStatus.Executed]: {
    text: 'Executed',
    color: 'success',
  },
  [TransactionStatus.ExecutionFailure]: {
    text: 'Execution Failure',
    color: 'error',
  },
  [TransactionStatus.MiscellaneousError]: {
    text: 'Miscellaneous Error',
    color: 'error',
  },
  [TransactionStatus.MoveAbort]: {
    text: 'Move Abort',
    color: 'error',
  },
  [TransactionStatus.OutOfGas]: {
    text: 'Out Of Gas',
    color: 'error',
  },
};

export const TRANSACTION_TYPE_MAP: Record<
  TransactionType,
  {
    text: ReactNode;
    color: ChipProps['color'];
  }
> = {
  [TransactionType.L1_BLOCK]: {
    text: 'L1 Block',
    color: 'success',
  },
  [TransactionType.L1_TX]: {
    text: 'L1 TX',
    color: 'secondary',
  },
  [TransactionType.L2_TX]: {
    text: 'L2 TX',
    color: 'error',
  },
};

export const TRANSACTION_ACTION_TYPE_MAP: Record<
  TransactionAction,
  {
    text: ReactNode;
    color: ChipProps['color'];
  }
> = {
  [TransactionAction.FunctionCall]: {
    text: 'Function Call',
    color: 'default',
  },
  [TransactionAction.ModuleBundle]: {
    text: 'Module Bundle',
    color: 'primary',
  },
  [TransactionAction.ScriptCall]: {
    text: 'Script Call',
    color: 'success',
  },
};
