import { gray } from '@/colors';
import { Box } from '@mui/material';
import SwapInfoCard, { SwapInfoCardProps } from './swap-info-card';

export interface SwapTransactionInfoCardProps extends SwapInfoCardProps {
  type: 'pending' | 'history';
}

export default function SwapTransactionInfoCard({
  fromCoin,
  toCoin,
  interactiveMode,
  loading,
  type,
}: SwapTransactionInfoCardProps) {
  return (
    <Box
      sx={{
        padding: '10px 20px',
        borderRadius: '8px',
        background: gray[50],
      }}
    >
      <SwapInfoCard
        fromCoin={fromCoin}
        toCoin={toCoin}
        interactiveMode={interactiveMode}
        loading={loading}
        type={type}
      />
    </Box>
  );
}
