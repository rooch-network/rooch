// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Next Imports
import Link from 'next/link'

// ** MUI Imports
import Box from '@mui/material/Box'
import { Theme } from '@mui/material/styles'
import { styled } from '@mui/material/styles'
import Typography from '@mui/material/Typography'
import useMediaQuery from '@mui/material/useMediaQuery'

const LinkStyled = styled(Link)(({ theme }) => ({
  textDecoration: 'none',
  color: theme.palette.primary.main,
}))

const FooterContent = () => {
  // ** Var
  const hidden = useMediaQuery((theme: Theme) => theme.breakpoints.down('md'))

  return (
    <Box
      sx={{
        display: 'flex',
        flexWrap: 'wrap',
        alignItems: 'center',
        justifyContent: 'space-between',
      }}
    >
      <Typography sx={{ mr: 2 }}>
        <LinkStyled target="_blank" href="https://rooch.network">
          © Root Branch Ltd.
        </LinkStyled>
        {` ${new Date().getFullYear()}. All rights reserved. `}
      </Typography>
      {hidden ? null : (
        <Box
          sx={{
            display: 'flex',
            flexWrap: 'wrap',
            alignItems: 'center',
            '& :not(:last-child)': { mr: 4 },
          }}
        >
          <LinkStyled target="_blank" href="https://github.com/rooch-network/">
            Github
          </LinkStyled>
          <LinkStyled target="_blank" href="https://discord.com/invite/rooch">
            Discord
          </LinkStyled>
          <LinkStyled target="_blank" href="https://twitter.com/RoochNetwork">
            Twitter
          </LinkStyled>
          <LinkStyled target="_blank" href="https://medium.com/rooch-network/">
            Medium
          </LinkStyled>
        </Box>
      )}
    </Box>
  )
}

export default FooterContent
