// ** Type Import
import { OwnerStateThemeType } from './'

// ** Util Import
import { hexToRGBA } from 'src/@core/utils/hex-to-rgba'

const Timeline = () => {
  return {
    MuiTimelineItem: {
      styleOverrides: {
        root: ({ theme }: OwnerStateThemeType) => ({
          '&:not(:last-of-type)': {
            '& .MuiTimelineContent-root': {
              marginBottom: theme.spacing(4)
            }
          }
        })
      }
    },
    MuiTimelineConnector: {
      styleOverrides: {
        root: ({ theme }: OwnerStateThemeType) => ({
          backgroundColor: theme.palette.divider
        })
      }
    },
    MuiTimelineContent: {
      styleOverrides: {
        root: ({ theme }: OwnerStateThemeType) => ({
          marginTop: theme.spacing(0.5)
        })
      }
    },
    MuiTimelineDot: {
      styleOverrides: {
        filledPrimary: ({ theme }: OwnerStateThemeType) => ({
          boxShadow: `0 0 0 4px ${hexToRGBA(theme.palette.primary.main, 0.16)}`
        }),
        filledSecondary: ({ theme }: OwnerStateThemeType) => ({
          boxShadow: `0 0 0 4px ${hexToRGBA(theme.palette.secondary.main, 0.16)}`
        }),
        filledSuccess: ({ theme }: OwnerStateThemeType) => ({
          boxShadow: `0 0 0 4px ${hexToRGBA(theme.palette.success.main, 0.16)}`
        }),
        filledError: ({ theme }: OwnerStateThemeType) => ({
          boxShadow: `0 0 0 4px ${hexToRGBA(theme.palette.error.main, 0.16)}`
        }),
        filledWarning: ({ theme }: OwnerStateThemeType) => ({
          boxShadow: `0 0 0 4px ${hexToRGBA(theme.palette.warning.main, 0.16)}`
        }),
        filledInfo: ({ theme }: OwnerStateThemeType) => ({
          boxShadow: `0 0 0 4px ${hexToRGBA(theme.palette.info.main, 0.16)}`
        }),
        filledGrey: ({ theme }: OwnerStateThemeType) => ({
          boxShadow: `0 0 0 4px ${hexToRGBA(theme.palette.grey[400], 0.16)}`
        }),
        outlinedPrimary: ({ theme }: OwnerStateThemeType) => ({
          '& svg': { color: theme.palette.primary.main }
        }),
        outlinedSecondary: ({ theme }: OwnerStateThemeType) => ({
          '& svg': { color: theme.palette.secondary.main }
        }),
        outlinedSuccess: ({ theme }: OwnerStateThemeType) => ({
          '& svg': { color: theme.palette.success.main }
        }),
        outlinedError: ({ theme }: OwnerStateThemeType) => ({
          '& svg': { color: theme.palette.error.main }
        }),
        outlinedWarning: ({ theme }: OwnerStateThemeType) => ({
          '& svg': { color: theme.palette.warning.main }
        }),
        outlinedInfo: ({ theme }: OwnerStateThemeType) => ({
          '& svg': { color: theme.palette.info.main }
        }),
        outlinedGrey: ({ theme }: OwnerStateThemeType) => ({
          '& svg': { color: theme.palette.grey[400] }
        })
      }
    }
  }
}

export default Timeline
