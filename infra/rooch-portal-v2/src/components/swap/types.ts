import type { ReactNode } from 'react';

export const DEFAULT_SLIPPAGE = 0.005;
export const VERSION_0 = 0;
export const VERSION_0_5 = 0.5;

export type SimulationStatus = 'new' | 'simulating' | 'success' | 'error';

/**
 * Common coin info type
 */
export interface Coin {
  coinType: string;
  decimals: number;
  name: string;
  symbol: string;
  icon: string;
  balance: bigint;
  price: number;
}

export interface UserCoin extends Coin {
  amount: bigint;
}

export type InteractiveMode = 'from' | 'to';

export type CurveType = 'stable' | 'uncorrelated';

export type PriceImpactSeverity = 'normal' | 'warning' | 'alert';

export type PoolVersion = typeof VERSION_0 | typeof VERSION_0_5;

export interface SwapProps {
  fixedSwap?: boolean;
  hiddenValue?: boolean;
  txHash?: string;
  /**
   * Current supported token list
   */
  coins: UserCoin[];

  /**
   * General loading status
   */
  loading?: boolean;

  /**
   * Current from currency
   */
  fromCoin?: UserCoin;

  /**
   * Current to currency
   */
  toCoin?: UserCoin;

  /**
   * Swap direction
   */
  interactiveMode: InteractiveMode;

  /**
   * Target swap amount
   */
  swapAmount?: bigint;

  /**
   * Slippage percentage
   */
  slippagePercent?: number;

  /**
   * Slippage amount
   */
  slippageAmount?: number;

  /**
   * Boolean flag indicating whether current pool exists or not
   */
  isPoolExist?: boolean;

  /**
   * Convert rate between from and to currency
   */
  convertRate?: number;

  /**
   * Platform fee percent
   */
  platformFeePercent?: number;

  /**
   * Platform fee amount
   */
  platformFeeAmount?: number;

  /**
   * Price impact percentage
   */
  priceImpact?: number;

  /**
   * Price impact severity
   */
  priceImpactSeverity?: PriceImpactSeverity;

  /**
   * Can select curve type
   */
  canSelectCurve?: boolean;

  /**
   * Swap curve type
   */
  curve?: CurveType;

  /**
   * Can select pool contract version
   */
  canSelectVersion?: boolean;

  /**
   * Contract version
   */
  version?: PoolVersion;

  /**
   * Swap error message
   */
  warning?: ReactNode;

  /**
   * Disable propose button and display error text
   */
  validationError?: string;

  isValid?: boolean
  /**
   * Gas info
   */
  gasInfo?: ReactNode;

  simulationStatus?: SimulationStatus;
  simulationError?: string;
  proposing?: boolean;

  /**
   * Callback for slippage change
   * @param slippage slippage
   * @returns
   */
  onSlippageChange: (slippage: number) => void;

  /**
   * Callback for curve type change
   * @param curveType curve type
   * @returns
   */
  onCurveTypeChange: (curveType: CurveType) => void;

  /**
   * Callback for version change
   * @param version pool contract version
   * @returns
   */
  onVersionChange: (version: PoolVersion) => void;

  /**
   * On switch token
   * @returns
   */
  onSwitch?: () => void;

  /**
   * On swap parameter change
   * @param payload swap payload
   * @returns
   */
  onSwap: (payload: Pick<SwapProps, 'fromCoin' | 'toCoin' | 'interactiveMode'>) => Promise<void>;

  /**
   * On preview modal open
   * @returns
   */
  onPreview: () => Promise<void>;

  /**
   * On propose event
   * @returns
   */
  onPropose: () => Promise<void>;
}
