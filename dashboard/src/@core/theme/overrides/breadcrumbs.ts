// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { OwnerStateThemeType } from './'

const Breadcrumbs = () => {
  return {
    MuiBreadcrumbs: {
      styleOverrides: {
        root: ({ theme }: OwnerStateThemeType) => ({
          '& a': {
            textDecoration: 'none',
            color: theme.palette.primary.main,
          },
        }),
        li: ({ theme }: OwnerStateThemeType) => ({
          color: theme.palette.text.secondary,
          '& .MuiTypography-root': {
            color: 'inherit',
          },
        }),
      },
    },
  }
}

export default Breadcrumbs
