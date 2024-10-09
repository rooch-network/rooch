import { useMemo, useState } from 'react';

import { LoadingButton } from '@mui/lab';
import { Alert, Stack, darken } from '@mui/material';

import { grey, secondary } from 'src/theme/core';

import SwapHeader from './swap-header';
import SwapDetails from './swap-details';
import { DEFAULT_SLIPPAGE } from './types';
import SwapCoinInput from './swap-coin-input';
// import SwapPreviewModal from './swap-preview-modal';
import SwapSwitchIcon from './swap-switch-icon';
import CurveTypeSelect from './curve-type-select';

import type { SwapProps } from './types';

export default function Swap({
  loading,
  coins,
  fromCoin,
  toCoin,
  interactiveMode,
  canSelectCurve,
  swapAmount,
  convertRate,
  slippagePercent = DEFAULT_SLIPPAGE,
  slippageAmount,
  platformFeePercent,
  platformFeeAmount,
  priceImpact,
  priceImpactSeverity,
  curve,
  warning,
  validationError,
  canSelectVersion,
  version,
  gasInfo,
  simulationStatus,
  simulationError,
  proposing,
  onSlippageChange,
  onCurveTypeChange,
  onVersionChange,
  onSwitch,
  onSwap,
  onPreview,
  onPropose,
}: SwapProps) {
  const [openPreview, setOpenPreview] = useState(false);

  const disabledCoins: string[] = useMemo(
    () => [fromCoin?.coinType || '', toCoin?.coinType || ''],
    [fromCoin?.coinType, toCoin?.coinType]
  );

  const showDetails = useMemo(
    () =>
      !!(
        fromCoin?.coinType &&
        fromCoin?.amount &&
        toCoin?.coinType &&
        toCoin?.amount &&
        interactiveMode
      ),
    [fromCoin?.coinType, fromCoin?.amount, toCoin?.coinType, toCoin?.amount, interactiveMode]
  );

  const proposeButtonContent: { text: string; disabled?: boolean } = useMemo(() => {
    if (validationError) {
      return {
        text: validationError,
        disabled: true,
      };
    }

    if ((fromCoin?.amount || 0) > (fromCoin?.balance || 0)) {
      return {
        text: 'Insufficient balance',
        disabled: true,
      };
    }

    if (slippagePercent <= 0 || slippagePercent > 0.5) {
      return {
        text: 'Invalid slippage',
        disabled: true,
      };
    }

    if (showDetails) {
      return {
        text: 'Propose',
      };
    }
    return {
      text: 'Propose',
      disabled: true,
    };
  }, [validationError, fromCoin?.amount, fromCoin?.balance, slippagePercent, showDetails]);

  const sortedBalanceCoins = useMemo(
    () =>
      coins.sort((a, b) => {
        if (a.balance === 0n) {
          return 1;
        }
        return -1;
      }),
    [coins]
  );

  return (
    <Stack
      direction="column"
      sx={{
        border: `1px solid ${grey[200]}`,
        borderRadius: '16px',
        boxShadow: '0px 5px 40px 0px rgba(16, 16, 40, 0.10)',
      }}
    >
      <SwapHeader slippage={slippagePercent} onSlippageChange={onSlippageChange} />
      <Stack spacing={3} padding={4}>
        <Stack spacing={-1} alignItems="center">
          <SwapCoinInput
            coins={coins}
            coin={fromCoin}
            type="from"
            interactiveMode={interactiveMode}
            disabledCoins={disabledCoins}
            onChange={(coin, source) => {
              onSwap({
                fromCoin: coin,
                toCoin,
                interactiveMode: source === 'amount' ? 'from' : 'to',
              });
            }}
          />
          <SwapSwitchIcon onClick={onSwitch} />
          <SwapCoinInput
            coins={sortedBalanceCoins}
            coin={toCoin}
            type="to"
            interactiveMode={interactiveMode}
            disabledCoins={disabledCoins}
            onChange={(coin, source) => {
              onSwap({
                fromCoin,
                toCoin: coin,
                interactiveMode: source === 'amount' ? 'to' : 'from',
              });
            }}
          />
        </Stack>

        {warning &&
          (typeof warning === 'string' ? (
            <Alert color="warning" severity="error">
              {warning}
            </Alert>
          ) : (
            warning
          ))}

        {canSelectCurve && curve && (
          <CurveTypeSelect curveType={curve} onChange={onCurveTypeChange} />
        )}
        {showDetails && (
          <SwapDetails
            loading={loading}
            fromCoin={fromCoin}
            toCoin={toCoin}
            interactiveMode={interactiveMode}
            swapAmount={swapAmount}
            slippagePercent={slippagePercent}
            slippageAmount={slippageAmount}
            platformFeePercent={platformFeePercent}
            platformFeeAmount={platformFeeAmount}
            convertRate={convertRate}
            priceImpact={priceImpact}
            priceImpactSeverity={priceImpactSeverity}
            canSelectCurve={canSelectCurve}
            curve={curve}
            canSelectVersion={canSelectVersion}
            version={version}
            variant="propose"
            onVersionChange={onVersionChange}
          />
        )}
        <LoadingButton
          color="primary"
          variant="contained"
          loading={loading}
          disabled={proposeButtonContent.disabled}
          sx={{
            background: secondary.light,
            height: '52px',
            '&:hover': { background: darken(secondary.light, 0.2) },
          }}
          onClick={() => {
            setOpenPreview(true);
            onPreview();
          }}
        >
          {proposeButtonContent.text}
        </LoadingButton>
      </Stack>
      {/* <SwapPreviewModal
        open={openPreview}
        onClose={() => setOpenPreview(false)}
        fromCoin={fromCoin}
        toCoin={toCoin}
        interactiveMode={interactiveMode}
        swapAmount={swapAmount}
        slippagePercent={slippagePercent}
        slippageAmount={slippageAmount}
        platformFeePercent={platformFeePercent}
        platformFeeAmount={platformFeeAmount}
        convertRate={convertRate}
        priceImpact={priceImpact}
        priceImpactSeverity={priceImpactSeverity}
        canSelectCurve={canSelectCurve}
        curve={curve}
        canSelectVersion={canSelectVersion}
        version={version}
        gasInfo={gasInfo}
        simulationStatus={simulationStatus}
        simulationError={simulationError}
        proposing={proposing}
        onPropose={onPropose}
      /> */}
    </Stack>
  );
}
