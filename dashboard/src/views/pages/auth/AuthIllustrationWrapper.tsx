// ** MUI Components
import { styled } from '@mui/material/styles'
import Box, { BoxProps } from '@mui/material/Box'

// ** Styled Components
const AuthIllustrationWrapper = styled(Box)<BoxProps>(({ theme }) => ({
  width: '100%',
  maxWidth: 400,
  position: 'relative',
  [theme.breakpoints.up('md')]: {
    '&:before': {
      zIndex: -1,
      top: '-40px',
      content: '""',
      right: '-40px',
      width: '148px',
      height: '148px',
      position: 'absolute',
      backgroundImage: `url(/images/pages/auth-illustration-top.png)`
    },
    '&:after': {
      zIndex: -1,
      left: '-46px',
      content: '""',
      width: '243px',
      height: '240px',
      bottom: '-68px',
      position: 'absolute',
      backgroundImage: `url(/images/pages/auth-illustration-bottom-${theme.palette.mode}.png)`
    }
  }
}))

export default AuthIllustrationWrapper
