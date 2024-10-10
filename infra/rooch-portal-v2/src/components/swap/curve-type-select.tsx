import { Stack, Button, Tooltip } from '@mui/material';

import { Iconify } from '../iconify';

import type { CurveType } from './types';

export interface CurveTypeSelectProps {
  curveType: CurveType;
  onChange: (curveType: CurveType) => void;
}

export default function CurveTypeSelect({ curveType, onChange }: CurveTypeSelectProps) {
  return (
    <Stack direction="row" spacing={2} alignItems="center">
      <OptionButton
        curveType={curveType}
        value="uncorrelated"
        icon="assets/icons/swap/relation-uncorrelated.svg"
        tooltip="Using x*y=K formula"
        onClick={() => onChange('uncorrelated')}
      />
      <OptionButton
        curveType={curveType}
        value="stable"
        icon="assets/icons/swap/relation-stable.svg"
        tooltip="Using formula optimized for stabled tokens swaps"
        onClick={() => onChange('stable')}
      />
    </Stack>
  );
}

function OptionButton({
  curveType,
  value,
  icon,
  tooltip,
  onClick,
}: {
  curveType: CurveType;
  value: CurveType;
  icon: string;
  tooltip: string;
  onClick: (curveType: CurveType) => void;
}) {
  return (
    <Button
      variant="outlined"
      color={curveType === value ? 'primary' : 'secondary'}
      startIcon={<img src={icon} alt="curve" />}
      endIcon={
        <Tooltip title={tooltip}>
          <Iconify icon="solar:question-circle-outline" width="20px" />
        </Tooltip>
      }
      sx={{ width: '50%' }}
      onClick={() => onClick(value)}
    >
      {value.charAt(0).toUpperCase() + value.slice(1)}
    </Button>
  );
}
