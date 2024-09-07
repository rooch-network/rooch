import type { Transition } from 'framer-motion';
import type { BoxProps } from '@mui/material/Box';
import type { AvatarProps } from '@mui/material/Avatar';

import { m } from 'framer-motion';

import Box from '@mui/material/Box';
import Avatar from '@mui/material/Avatar';

export type AnimateAvatarProps = BoxProps & {
  slotProps?: {
    avatar?: AvatarProps;
    animate?: { transition?: Transition };
    overlay?: {
      color?: string;
      border?: number;
      spacing?: number;
    };
  };
};

export function AnimateAvatar({
  sx,
  slotProps,
  children,
  width = 40,
  ...other
}: AnimateAvatarProps) {
  const borderWidth = slotProps?.overlay?.border ?? 2;

  const spacing = slotProps?.overlay?.spacing ?? 2;

  return (
    <Box
      sx={{
        width,
        height: width,
        flexShrink: 0,
        borderRadius: '50%',
        position: 'relative',
        alignItems: 'center',
        display: 'inline-flex',
        justifyContent: 'center',
        ...sx,
      }}
      {...other}
    >
      <Avatar
        alt={slotProps?.avatar?.alt ?? 'My avtar'}
        src={slotProps?.avatar?.src}
        sx={{
          zIndex: 1,
          width: `calc(100% - ${borderWidth * 2 + spacing * 2}px)`,
          height: `calc(100% - ${borderWidth * 2 + spacing * 2}px)`,
          ...slotProps?.avatar?.sx,
        }}
        {...slotProps?.avatar}
      >
        {children}
      </Avatar>

      <Box
        component={m.span}
        animate={{ rotate: 360 }}
        transition={{
          duration: 2,
          ease: 'linear',
          repeat: Infinity,
          ...slotProps?.animate?.transition,
        }}
        sx={{
          top: 0,
          left: 0,
          width: 1,
          height: 1,
          position: 'absolute',
          borderRadius: 'inherit',
          background: slotProps?.overlay?.color ?? 'conic-gradient(cyan, magenta, yellow, cyan)',
          mask: 'linear-gradient(#FFF 0 0) content-box, linear-gradient(#FFF 0 0)',
          WebkitMask: 'linear-gradient(#FFF 0 0) content-box, linear-gradient(#FFF 0 0)',
          maskComposite: 'exclude',
          WebkitMaskComposite: 'xor',
          p: `${borderWidth}px`,
        }}
      />
    </Box>
  );
}
