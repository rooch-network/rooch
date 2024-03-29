// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { OwnerStateThemeType } from './'

const Progress = () => {
  return {
    MuiLinearProgress: {
      styleOverrides: {
        root: ({ theme }: OwnerStateThemeType) => ({
          height: 12,
          borderRadius: 10,
          backgroundColor: theme.palette.customColors.trackBg,
        }),
        bar: {
          borderRadius: 10,
        },
      },
    },
  }
}

export default Progress
