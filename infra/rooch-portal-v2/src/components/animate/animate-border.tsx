'use client';

import type { BoxProps } from '@mui/material/Box';
import type { Easing, RepeatType } from 'framer-motion';

import { m } from 'framer-motion';
import { useRef, useState, useEffect } from 'react';

import Box from '@mui/material/Box';

import { borderGradient } from 'src/theme/styles';

export type AnimateBorderProps = BoxProps & {
  animate?: {
    outline?: string;
    color?: string | string[];
    width?: string; // width `2px` | `2px 4px 0 0` (as padding)
    angle?: number; // angle: min: 0, max: 360, step: 1
    loop?: boolean;
    length?: number; // length: min: 1, max: 100, step: 1
    distance?: number; // distance: min: 1, max: 100, step: 1
    ease?: Easing;
    delay?: number;
    duration?: number; // duration: min: 1, max: 20, step: 1
    repeatType?: RepeatType; // repeatType: ["loop", "reverse", "mirror"
    disable?: boolean; // disable animate
    disableDoubleline?: boolean; // show 1 line
  };
};

export function AnimateBorder({ animate, sx }: AnimateBorderProps) {
  const rootRef = useRef<HTMLDivElement | null>(null);

  const animateRef = useRef<HTMLDivElement | null>(null);

  const [aspectRatio, setAspectRatio] = useState(1);

  const [animateStyle, setAnimateStyle] = useState<React.CSSProperties | null>(null);

  const values = {
    disable: animate?.disable,
    delay: animate?.delay ?? 0,
    loop: animate?.loop ?? true,
    angle: animate?.angle ?? 315,
    length: animate?.length ?? 40,
    width: animate?.width ?? '2px',
    color: animate?.color ?? '#000',
    ease: animate?.ease ?? 'linear',
    duration: animate?.duration ?? 8,
    distance: animate?.distance ?? 20,
    repeatType: animate?.repeatType ?? 'loop',
    disableDoubleline: animate?.disableDoubleline,
    outline: animate?.outline ?? `135deg, rgba(0,0,0,0.08), rgba(0,0,0,0.08)`,
  };

  useEffect(() => {
    if (!values.disable) {
      if (rootRef.current) {
        const { width, height } = rootRef.current.getBoundingClientRect();

        setAspectRatio(width / height);
      }

      if (!values.disableDoubleline && animateRef.current) {
        const style = getComputedStyle(animateRef.current);

        setAnimateStyle({
          paddingLeft: style.paddingLeft,
          paddingRight: style.paddingRight,
          paddingBottom: style.paddingBottom,
          paddingTop: style.paddingTop,
          borderTopLeftRadius: style.borderTopLeftRadius,
          borderTopRightRadius: style.borderTopRightRadius,
          borderBottomLeftRadius: style.borderBottomLeftRadius,
          borderBottomRightRadius: style.borderBottomRightRadius,
        });
      }
    }
  }, [values.disable, values.disableDoubleline]);

  const background = (color: string) => {
    const degs = [-55, 35, 125, 215, 305];

    const end = `transparent ${values.angle - (2 + values.length!)}deg, ${color}  ${values.angle}deg, transparent ${values.angle + values.length}deg`;

    return [
      `conic-gradient(from ${degs[0]}deg at ${values.distance! / aspectRatio}% ${values.distance}% , ${end})`,
      `conic-gradient(from ${degs[1]}deg at ${100 - values.distance! / aspectRatio}% ${values.distance}% , ${end})`,
      `conic-gradient(from ${degs[2]}deg at ${100 - values.distance / aspectRatio}% ${100 - values.distance}% , ${end})`,
      `conic-gradient(from ${degs[3]}deg at ${values.distance / aspectRatio}% ${100 - values.distance}% , ${end})`,
      `conic-gradient(from ${degs[4]}deg at ${values.distance / aspectRatio}% ${values.distance}% , ${end})`,
    ];
  };

  const transition = {
    ease: values.ease,
    delay: values.delay,
    duration: values.duration,
    repeatType: values.repeatType,
    repeat: values.loop ? Infinity : 1,
    times:
      aspectRatio > 1
        ? [0, 0.25 + 0.25 / aspectRatio, 0.5, 0.75 + 0.25 / aspectRatio, 1]
        : [0, aspectRatio * 0.25, 0.5, 0.5 + aspectRatio * 0.25, 1],
  };

  return (
    <Box
      ref={rootRef}
      sx={{
        minWidth: 40,
        minHeight: 40,
        position: 'relative',
        borderRadius: 'inherit',
        '&::before': {
          ...borderGradient({ color: values.outline, padding: values.width }),
        },
        ...sx,
      }}
    >
      <Box
        ref={animateRef}
        component={m.span}
        transition={transition}
        animate={
          !values.disable
            ? {
                background: background(
                  typeof values.color === 'string' ? values.color : values.color[0]
                ),
              }
            : undefined
        }
        sx={{
          ...borderGradient({ padding: values.width }),
        }}
      />

      {!values.disable && !values.disableDoubleline && (
        <Box
          component={m.span}
          transition={transition}
          animate={{
            background: background(
              typeof values.color === 'string' ? values.color : values.color[1]
            ),
          }}
          sx={{
            ...borderGradient(),
            transform: 'scale(-1)',
            ...(animateStyle && {
              paddingTop: animateStyle?.paddingBottom,
              paddingBottom: animateStyle?.paddingTop,
              paddingLeft: animateStyle?.paddingRight,
              paddingRight: animateStyle?.paddingLeft,
              borderTopLeftRadius: animateStyle?.borderBottomRightRadius,
              borderTopRightRadius: animateStyle?.borderBottomLeftRadius,
              borderBottomLeftRadius: animateStyle?.borderTopRightRadius,
              borderBottomRightRadius: animateStyle?.borderTopLeftRadius,
            }),
          }}
        />
      )}
    </Box>
  );
}
