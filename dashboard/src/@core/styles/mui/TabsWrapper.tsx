// ** React Import
import { ReactNode } from 'react'

// ** MUI imports
import { styled } from '@mui/material/styles'
import Box, { BoxProps } from '@mui/material/Box'

type Props = {
  children: ReactNode
} & (
  | {
      orientation?: 'vertical'
      panelLeftRound?: 'top' | 'bottom' | 'both'
      panelTopRound?: never
    }
  | {
      orientation?: 'horizontal' | never
      panelTopRound?: 'left' | 'right' | 'both'
      panelLeftRound?: never
    }
)

const TabsWrapper = ({ children, orientation, panelTopRound, panelLeftRound }: Props) => {
  const Wrapper = styled(Box)<BoxProps>(({ theme }) => ({
    '& .MuiTabPanel-root': {
      borderBottomRightRadius: theme.shape.borderRadius,
      ...(orientation !== 'vertical' && {
        borderBottomLeftRadius: theme.shape.borderRadius,
        ...(panelTopRound === 'left' && { borderTopLeftRadius: theme.shape.borderRadius }),
        ...(panelTopRound === 'right' && { borderTopRightRadius: theme.shape.borderRadius }),
        ...(panelTopRound === 'both' && {
          borderTopLeftRadius: theme.shape.borderRadius,
          borderTopRightRadius: theme.shape.borderRadius
        })
      }),
      ...(orientation === 'vertical' && {
        borderTopRightRadius: theme.shape.borderRadius,
        ...(panelLeftRound === 'top' && { borderTopLeftRadius: theme.shape.borderRadius }),
        ...(panelLeftRound === 'bottom' && { borderBottomLeftRadius: theme.shape.borderRadius }),
        ...(panelLeftRound === 'both' && {
          borderTopLeftRadius: theme.shape.borderRadius,
          borderBottomLeftRadius: theme.shape.borderRadius
        })
      })
    }
  }))

  return <Wrapper>{children}</Wrapper>
}

export default TabsWrapper
