import type { FabProps } from '@mui/material/Fab';

import { useState } from 'react';
import { useScroll, useMotionValueEvent } from 'framer-motion';

import Fab from '@mui/material/Fab';

import { Iconify } from 'src/components/iconify';

export type BackToTopProps = FabProps & {
  value?: number;
};

export function BackToTop({ value = 90, sx, ...other }: BackToTopProps) {
  const { scrollYProgress } = useScroll();

  const [show, setShow] = useState<boolean>(false);

  const backToTop = () => {
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  useMotionValueEvent(scrollYProgress, 'change', (latest) => {
    const isEnd = Math.floor(latest * 100) > value; // unit is %
    setShow(isEnd);
  });

  return (
    <Fab
      aria-label="Back to top"
      onClick={backToTop}
      sx={{
        width: 48,
        height: 48,
        position: 'fixed',
        transform: 'scale(0)',
        right: { xs: 24, md: 32 },
        bottom: { xs: 24, md: 32 },
        zIndex: (theme) => theme.zIndex.speedDial,
        transition: (theme) => theme.transitions.create(['transform']),
        ...(show && { transform: 'scale(1)' }),
        ...sx,
      }}
      {...other}
    >
      <Iconify width={24} icon="solar:double-alt-arrow-up-bold-duotone" />
    </Fab>
  );
}
