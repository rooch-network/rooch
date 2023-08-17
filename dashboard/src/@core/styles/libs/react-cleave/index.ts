// ** MUI Imports
import { styled } from '@mui/material/styles'
import Box, { BoxProps } from '@mui/material/Box'

// ** Util Import
import { hexToRGBA } from 'src/@core/utils/hex-to-rgba'

const CleaveWrapper = styled(Box)<BoxProps>(({ theme }) => ({
  '& input': {
    height: 56,
    fontSize: 16,
    width: '100%',
    borderWidth: 1,
    background: 'none',
    borderStyle: 'solid',
    padding: '16.5px 14px',
    color: theme.palette.text.primary,
    borderRadius: theme.shape.borderRadius,
    fontFamily: theme.typography.body1.fontFamily,
    borderColor: `rgba(${theme.palette.customColors.main}, 0.22)`,
    transition: theme.transitions.create(['border-color', 'box-shadow']),
    '&:focus, &:focus-visible': {
      outline: 0,
      borderWidth: 2,
      padding: '15.5px 13px',
      borderColor: `${theme.palette.primary.main} !important`,
      boxShadow: `0 1px 3px 0 ${hexToRGBA(theme.palette.primary.main, 0.4)}`
    },
    '&::-webkit-input-placeholder': {
      color: theme.palette.text.secondary
    }
  }
}))

export default CleaveWrapper
