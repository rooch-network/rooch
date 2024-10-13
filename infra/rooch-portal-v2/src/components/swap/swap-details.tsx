import type { ReactNode } from 'react';

import { useMemo, useState } from 'react';

import {
  Box,
  Stack,
  Divider,
  Accordion,
  Typography,
  AccordionDetails,
  AccordionSummary,
  CircularProgress,
} from '@mui/material';

import { fNumber } from 'src/utils/format-number';

import { grey, error, success, warning } from 'src/theme/core';

import Text from './typography/text';
import { Iconify } from '../iconify';
import Label from './typography/label';
import PoolVersionSelect from './pool-version-select';
import { toBigNumber, formatCurrency, fromDustToPrecision } from '../../utils/number';

import type { SwapProps, PoolVersion, PriceImpactSeverity } from './types';

export default function SwapDetails({
  loading,
  interactiveMode,
  fromCoin,
  toCoin,
  swapAmount,
  convertRate,
  slippagePercent,
  slippageAmount,
  platformFeePercent,
  platformFeeAmount,
  priceImpact,
  priceImpactSeverity,
  canSelectCurve,
  curve,
  canSelectVersion,
  version,
  variant,
  fixedSwap,
  onVersionChange,
}: Omit<
  SwapProps,
  | 'coins'
  | 'onSlippageChange'
  | 'onCurveTypeChange'
  | 'onVersionChange'
  | 'getUsdEquivalent'
  | 'onSwap'
  | 'onSwitch'
  | 'onPreview'
  | 'onPropose'
> & { variant: 'propose' | 'transaction'; onVersionChange?: (version: PoolVersion) => void }) {
  const [showDetails, setShowDetails] = useState(false);

  const targetToken = useMemo(
    () => (interactiveMode === 'from' ? fromCoin : toCoin),
    [interactiveMode, fromCoin, toCoin]
  );

  const header = useMemo(() => {
    if (variant === 'propose') {
      return (
        <Typography
          sx={{
            fontSize: '0.875rem',
            fontWeight: 500,
            lineHeight: '24px',
            color: grey[900],
          }}
        >
          1 {fromCoin?.symbol} = {fNumber(toBigNumber(convertRate).toString())} {toCoin?.symbol}{' '}
          {!fixedSwap &&
            `(including
          fee)`}
        </Typography>
      );
    }
    return (
      <Stack sx={{ width: '100%' }}>
        {interactiveMode === 'from' && !fixedSwap && (
          <DetailsItem
            left={<Text>Expected Output</Text>}
            right={
              <Label>
                {fromDustToPrecision(swapAmount || 0, toCoin?.decimals || 1)} {toCoin?.symbol}
              </Label>
            }
          />
        )}
        {interactiveMode === 'to' && (
          <DetailsItem
            left={<Text>Expected Input</Text>}
            right={
              <Label>
                {fromDustToPrecision(swapAmount || 0, fromCoin?.decimals || 1)} {fromCoin?.symbol}
              </Label>
            }
          />
        )}
      </Stack>
    );
  }, [
    variant,
    interactiveMode,
    fixedSwap,
    swapAmount,
    toCoin?.decimals,
    toCoin?.symbol,
    fromCoin?.decimals,
    fromCoin?.symbol,
    convertRate,
  ]);

  return (
    <Stack spacing={1.5}>
      <Accordion
        disableGutters
        elevation={0}
        expanded={!loading}
        sx={{
          '&.MuiAccordion-root:before': { opacity: 0 },
          '& .MuiAccordionDetails-root': {
            transition: 'all 0.3s ease-in-out',
          },
        }}
      >
        <AccordionSummary
          expandIcon={
            fixedSwap ? null : (
              <Iconify
                icon="solar:alt-arrow-down-linear"
                style={{
                  transition: 'all 0.1s ease-in-out',
                  transform: showDetails ? '' : 'rotate(180deg)',
                }}
              />
            )
          }
          onClick={() => {
            if (fixedSwap) {
              return;
            }
            setShowDetails(!showDetails);
          }}
          sx={{
            borderRadius: '8px 8px 0 0',
            border: `1px solid ${grey[300]}`,
            padding: '4px 16px',
          }}
        >
          {loading ? <CircularProgress size="1.5rem" /> : header}
        </AccordionSummary>
        <AccordionDetails
          sx={{
            borderRadius: '0 0 8px 8px',
            border: `1px solid ${grey[300]}`,
            borderTop: 'none',
            padding: '18px 16px',
          }}
        >
          {interactiveMode === 'from' && (
            <Stack spacing={0.5}>
              {variant === 'propose' && (
                <DetailsItem
                  left={
                    <Text className="!text-gray-400">
                      Estimated rate, the actual amount received depends on the rate at the time of
                      transaction confirmation block.
                    </Text>
                  }
                  right={
                    fixedSwap ? null : (
                      <Label>
                        {`${fromDustToPrecision(swapAmount || 0, toCoin?.decimals || 1)} ${
                          toCoin?.symbol
                        }`}
                      </Label>
                    )
                  }
                />
              )}
              {!fixedSwap && (
                <DetailsItem
                  left={
                    <Text>Minimum Received after Slippage ({(slippagePercent || 0) * 100} %)</Text>
                  }
                  right={
                    <Label>
                      {`${formatCurrency(slippageAmount || 0, toCoin?.decimals || 1)} ${
                        toCoin?.symbol
                      }`}
                    </Label>
                  }
                />
              )}
            </Stack>
          )}
          {interactiveMode === 'to' && (
            <Stack spacing={0.5}>
              {variant === 'propose' && (
                <DetailsItem
                  left={<Text>Expected Input</Text>}
                  right={
                    <Label>
                      {`${fromDustToPrecision(swapAmount || 0, fromCoin?.decimals || 1)} ${
                        fromCoin?.symbol
                      }`}
                    </Label>
                  }
                />
              )}
              <DetailsItem
                left={<Text>Maximum Send after Slippage ({(slippagePercent || 0) * 100} %)</Text>}
                right={
                  <Label>
                    {`${formatCurrency(slippageAmount || 0, fromCoin?.decimals || 1)} ${
                      fromCoin?.symbol
                    }`}
                  </Label>
                }
              />
            </Stack>
          )}
          {showDetails && !fixedSwap && (
            <>
              <Divider sx={{ my: '10px' }} />
              <Stack spacing={0.5}>
                {variant === 'propose' && !fixedSwap && (
                  <DetailsItem
                    left={<Label>Price Impact</Label>}
                    right={
                      <PriceImpactLabel
                        priceImpact={(priceImpact || 0) * 100}
                        priceImpactSeverity={priceImpactSeverity || 'alert'}
                      />
                    }
                  />
                )}
                <DetailsItem
                  left={
                    <Label>
                      Platform Fee ({toBigNumber(platformFeePercent).times(100).toString()} %)
                    </Label>
                  }
                  right={
                    <Label>
                      {formatCurrency(platformFeeAmount || 0, targetToken?.decimals || 1)}{' '}
                      {targetToken?.symbol}
                    </Label>
                  }
                />
                {variant === 'propose' && !canSelectCurve && !fixedSwap && (
                  <DetailsItem left={<Label>Curve Type</Label>} right={<Label>{curve}</Label>} />
                )}
                {variant === 'propose' && canSelectVersion && !fixedSwap && (
                  <DetailsItem
                    left={<Label>Pool Version</Label>}
                    right={<PoolVersionSelect version={version} onChange={onVersionChange} />}
                  />
                )}
              </Stack>
            </>
          )}
        </AccordionDetails>
      </Accordion>
    </Stack>
  );
}

function DetailsItem({ left, right }: { left: ReactNode; right: ReactNode }) {
  return (
    <Stack direction="row" spacing={3} alignItems="center">
      <Box sx={{ flexGrow: 1 }}>{left}</Box>
      <Box sx={{ flexShrink: 0 }}>{right}</Box>
    </Stack>
  );
}

function PriceImpactLabel({
  priceImpact,
  priceImpactSeverity,
}: {
  priceImpact: number;
  priceImpactSeverity: PriceImpactSeverity;
}) {
  const color = useMemo(() => {
    if (priceImpactSeverity === 'normal') {
      return success[700];
    }
    if (priceImpactSeverity === 'warning') {
      return warning[500];
    }
    return error[800];
  }, [priceImpactSeverity]);

  return (
    <Stack
      direction="row"
      alignItems="center"
      justifyContent="center"
      spacing={1}
      sx={{
        padding: '4px 8px',
        borderRadius: '6px',
        background: color,
      }}
    >
      <img src="assets/icons/swap/alert-triangle.svg" alt="alert" />
      <Typography
        sx={{
          fontSize: '0.75rem',
          fontWeight: 500,
          lineHeight: '24px',
          color: '#fff',
        }}
      >
        {toBigNumber(priceImpact).toFixed(2)} %
      </Typography>
    </Stack>
  );
}
