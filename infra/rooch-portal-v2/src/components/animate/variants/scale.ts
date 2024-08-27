import { varTranExit, varTranEnter } from './transition';

import type { VariantsType } from '../types';

export const varScale = (props?: VariantsType) => {
  const durationIn = props?.durationIn;
  const durationOut = props?.durationOut;
  const easeIn = props?.easeIn;
  const easeOut = props?.easeOut;

  return {
    // IN
    in: {
      initial: { scale: 0, opacity: 0 },
      animate: { scale: 1, opacity: 1, transition: varTranEnter({ durationIn, easeIn }) },
      exit: { scale: 0, opacity: 0, transition: varTranExit({ durationOut, easeOut }) },
    },
    inX: {
      initial: { scaleX: 0, opacity: 0 },
      animate: { scaleX: 1, opacity: 1, transition: varTranEnter({ durationIn, easeIn }) },
      exit: { scaleX: 0, opacity: 0, transition: varTranExit({ durationOut, easeOut }) },
    },
    inY: {
      initial: { scaleY: 0, opacity: 0 },
      animate: { scaleY: 1, opacity: 1, transition: varTranEnter({ durationIn, easeIn }) },
      exit: { scaleY: 0, opacity: 0, transition: varTranExit({ durationOut, easeOut }) },
    },

    // OUT
    out: {
      initial: { scale: 1, opacity: 1 },
      animate: { scale: 0, opacity: 0, transition: varTranEnter({ durationIn, easeIn }) },
    },
    outX: {
      initial: { scaleX: 1, opacity: 1 },
      animate: { scaleX: 0, opacity: 0, transition: varTranEnter({ durationIn, easeIn }) },
    },
    outY: {
      initial: { scaleY: 1, opacity: 1 },
      animate: { scaleY: 0, opacity: 0, transition: varTranEnter({ durationIn, easeIn }) },
    },
  };
};
