'use client';

import type { MotionValue } from 'framer-motion';

import { useRef, useMemo } from 'react';
import { useScroll } from 'framer-motion';

export type UseScrollProgressReturn = {
  scrollXProgress: MotionValue<number>;
  scrollYProgress: MotionValue<number>;
  elementRef: React.RefObject<HTMLDivElement>;
};

export type UseScrollProgress = 'document' | 'container';

export function useScrollProgress(target: UseScrollProgress = 'document'): UseScrollProgressReturn {
  const elementRef = useRef<HTMLDivElement>(null);

  const options = { container: elementRef };

  const { scrollYProgress, scrollXProgress } = useScroll(
    target === 'container' ? options : undefined
  );

  const memoizedValue = useMemo(
    () => ({ elementRef, scrollXProgress, scrollYProgress }),
    [elementRef, scrollXProgress, scrollYProgress]
  );

  return memoizedValue;
}
