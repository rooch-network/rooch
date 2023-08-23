// ** Type Imports
import { OwnerStateThemeType } from './'
import { Skin } from 'src/@core/layouts/types'

const Snackbar = (skin: Skin) => {
  return {
    MuiSnackbarContent: {
      styleOverrides: {
        root: ({ theme }: OwnerStateThemeType) => ({
          ...(skin === 'bordered' && { boxShadow: 'none' }),
          backgroundColor: `rgb(${theme.palette.customColors.main})`,
          color: theme.palette.common[theme.palette.mode === 'light' ? 'white' : 'black']
        })
      }
    }
  }
}

export default Snackbar
