// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { OwnerStateThemeType } from './'

const Backdrop = () => {
  return {
    MuiBackdrop: {
      styleOverrides: {
        root: ({ theme }: OwnerStateThemeType) => ({
          backgroundColor:
            theme.palette.mode === 'light'
              ? `rgba(${theme.palette.customColors.main}, 0.5)`
              : 'rgba(14, 15, 36, 0.68)',
        }),
        invisible: {
          backgroundColor: 'transparent',
        },
      },
    },
  }
}

export default Backdrop
