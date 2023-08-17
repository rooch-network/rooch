// ** MUI Imports
import { Theme } from '@mui/material/styles'

// ** Type Imports
import { OwnerStateThemeType } from './'
import { Skin } from 'src/@core/layouts/types'

const Autocomplete = (skin: Skin) => {
  const boxShadow = (theme: Theme) => {
    if (skin === 'bordered') {
      return theme.shadows[0]
    } else if (theme.palette.mode === 'light') {
      return theme.shadows[8]
    } else return theme.shadows[9]
  }

  return {
    MuiAutocomplete: {
      styleOverrides: {
        paper: ({ theme }: OwnerStateThemeType) => ({
          boxShadow: boxShadow(theme),
          marginTop: theme.spacing(1),
          ...(skin === 'bordered' && { border: `1px solid ${theme.palette.divider}` })
        }),
        listbox: ({ theme }: OwnerStateThemeType) => ({
          padding: theme.spacing(1.25, 0),
          '& .MuiAutocomplete-option': {
            padding: theme.spacing(2, 5),
            '&[aria-selected="true"]': {
              color: theme.palette.primary.main
            }
          }
        })
      }
    }
  }
}

export default Autocomplete
