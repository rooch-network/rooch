// ** Type Import
import { OwnerStateThemeType } from './'

const List = () => {
  return {
    MuiListItemIcon: {
      styleOverrides: {
        root: ({ theme }: OwnerStateThemeType) => ({
          minWidth: '0 !important',
          marginRight: theme.spacing(2.25),
          color: theme.palette.text.secondary
        })
      }
    },
    MuiListItemAvatar: {
      styleOverrides: {
        root: ({ theme }: OwnerStateThemeType) => ({
          minWidth: 0,
          marginRight: theme.spacing(4)
        })
      }
    },
    MuiListItemText: {
      styleOverrides: {
        dense: ({ theme }: OwnerStateThemeType) => ({
          '& .MuiListItemText-primary': {
            color: theme.palette.text.primary
          }
        })
      }
    },
    MuiListSubheader: {
      styleOverrides: {
        root: ({ theme }: OwnerStateThemeType) => ({
          fontWeight: 600,
          textTransform: 'uppercase',
          color: theme.palette.text.primary
        })
      }
    }
  }
}

export default List
