import { Box } from '@mui/material';

import { secondary } from 'src/theme/core';

export interface SwapSwitchIconProps {
  onClick?: () => void;
}

export default function SwapSwitchIcon({ onClick }: SwapSwitchIconProps) {
  return (
    <Box
      component="button"
      className="flex justify-center items-center cursor-none"
      sx={{
        width: '32px',
        height: '32px',
        padding: '4px',
        borderRadius: '32px',
        border: '4px solid #FFF',
        background: secondary.light,
        cursor: 'pointer',
        transition: 'all 0.15s ease',
        zIndex: 1,
        // '&:hover': {
        //   transform: 'rotate(-180deg)',
        // },
      }}
      // onClick={onClick}
    >
      <Box
        component="img"
        src="assets/icons/swap/swap-down.svg"
        width="100%"
        className="ml-[1px]"
      />
    </Box>
  );
}
