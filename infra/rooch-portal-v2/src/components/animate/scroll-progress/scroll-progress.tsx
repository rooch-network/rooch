import type { MotionValue } from 'framer-motion';
import type { BoxProps } from '@mui/material/Box';

import { m, useSpring } from 'framer-motion';

import Box from '@mui/material/Box';

export interface ScrollProgressProps extends BoxProps {
  size?: number;
  thickness?: number;
  progress: MotionValue<number>;
  variant: 'linear' | 'circular';
  color?: 'inherit' | 'primary' | 'secondary' | 'info' | 'success' | 'warning' | 'error';
}

export function ScrollProgress({
  size,
  variant,
  progress,
  thickness = 3.6,
  color = 'primary',
  sx,
  ...other
}: ScrollProgressProps) {
  const scaleX = useSpring(progress, { stiffness: 100, damping: 30, restDelta: 0.001 });

  const progressSize = variant === 'circular' ? size ?? 64 : size ?? 3;

  const renderCircular = (
    <Box
      component="svg"
      width={progressSize}
      height={progressSize}
      viewBox={`0 0 ${progressSize} ${progressSize}`}
      xmlns="http://www.w3.org/2000/svg"
      sx={{
        width: progressSize,
        height: progressSize,
        transform: 'rotate(-90deg)',
        color: (theme) => theme.vars.palette.text.primary,
        ...(color !== 'inherit' && {
          color: (theme) => theme.vars.palette[color].main,
        }),
        circle: {
          fill: 'none',
          strokeDashoffset: 0,
          strokeWidth: thickness,
          stroke: 'currentColor',
        },
        ...sx,
      }}
      {...other}
    >
      <Box
        component="circle"
        cx={progressSize / 2}
        cy={progressSize / 2}
        r={progressSize / 2 - thickness - 4}
        strokeOpacity="0.2"
        pathLength="1"
      />
      <Box
        component={m.circle}
        cx={progressSize / 2}
        cy={progressSize / 2}
        r={progressSize / 2 - thickness - 4}
        pathLength="1"
        style={{ pathLength: progress }}
      />
    </Box>
  );

  const renderLinear = (
    <Box
      component={m.div}
      sx={{
        top: 0,
        left: 0,
        right: 0,
        zIndex: 1999,
        height: progressSize,
        transformOrigin: '0%',
        bgcolor: 'text.primary',
        ...(color !== 'inherit' && {
          background: (theme) =>
            `linear-gradient(135deg, ${theme.vars.palette[color].light}, ${theme.vars.palette[color].main})`,
        }),
        ...sx,
      }}
      style={{ scaleX }}
      {...other}
    />
  );

  return (
    <Box sx={{ overflow: 'hidden' }}>{variant === 'circular' ? renderCircular : renderLinear}</Box>
  );
}
