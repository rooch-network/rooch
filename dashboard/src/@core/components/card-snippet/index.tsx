// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { useState, useEffect } from 'react'

// ** MUI Imports
import Box from '@mui/material/Box'
import Card from '@mui/material/Card'
import Tooltip from '@mui/material/Tooltip'
import Divider from '@mui/material/Divider'
import { Theme } from '@mui/material/styles'
import Collapse from '@mui/material/Collapse'
import IconButton from '@mui/material/IconButton'
import CardHeader from '@mui/material/CardHeader'
import CardContent from '@mui/material/CardContent'
import ToggleButton from '@mui/material/ToggleButton'
import useMediaQuery from '@mui/material/useMediaQuery'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'

// ** Icon Imports
import Icon from 'src/@core/components/icon'

// ** Third Party Components
import Prism from 'prismjs'
import toast from 'react-hot-toast'

// ** Types
import { CardSnippetProps } from './types'

// ** Hooks
import useClipboard from 'src/@core/hooks/useClipboard'

// TODO: prismjs theme config
const CardSnippet = (props: CardSnippetProps) => {
  // ** Props
  const { id, sx, defaultShow, fullHeight, codes, title, children, className } = props

  // ** States
  const [showCode, setShowCode] = useState<boolean>(defaultShow ?? false)
  const [tabValue, setTabValue] = useState<number>(0)

  // ** Hooks
  const clipboard = useClipboard()
  const hidden = useMediaQuery((theme: Theme) => theme.breakpoints.down('md'))

  // ** Highlight code on mount
  useEffect(() => {
    Prism.highlightAll()
  }, [showCode, tabValue])

  const codeToCopy = () => {
    return codes[tabValue].code
  }

  const handleClick = () => {
    clipboard.copy(codeToCopy())
    toast.success('The source code has been copied to your clipboard.', {
      duration: 2000,
    })
  }

  const renderCode = () => {
    const code = codes[tabValue]
    const className = `language-${code.lng}`

    return (
      <div>
        <pre className={className}>
          <code className={className}>{code.code}</code>
        </pre>
      </div>
    )
  }

  return (
    <Card
      className={className}
      sx={{ '& .MuiCardHeader-action': { lineHeight: 0.8 }, ...sx }}
      id={id || `card-snippet--${title?.toLowerCase().replace(/ /g, '-') ?? 'default'}`}
    >
      {title ? (
        <CardHeader
          title={title}
          {...(hidden || defaultShow
            ? {}
            : {
                action: (
                  <IconButton onClick={() => setShowCode(!showCode)}>
                    <Icon icon="bx:code" fontSize={20} />
                  </IconButton>
                ),
              })}
        />
      ) : null}
      {children ? <CardContent>{children}</CardContent> : null}
      <Collapse in={showCode}>
        {title ? <Divider sx={{ my: '0 !important' }} /> : null}
        <CardContent
          sx={{
            position: 'relative',
            ...(fullHeight
              ? { '& pre': { m: '0 !important' } }
              : {
                  '& pre': {
                    m: '0 !important',
                    maxHeight: 500,
                  },
                }),
          }}
        >
          <Box sx={{ mb: 4, display: 'flex', alignItems: 'center', justifyContent: 'flex-end' }}>
            <ToggleButtonGroup
              exclusive
              size="small"
              color="primary"
              value={tabValue}
              onChange={(e, newValue) => (newValue !== null ? setTabValue(newValue) : null)}
            >
              {codes.map((v, i) => (
                <ToggleButton key={v.lng} value={i}>
                  {v.lng}
                </ToggleButton>
              ))}
            </ToggleButtonGroup>
          </Box>

          <Tooltip title="Copy the source" placement="top">
            <IconButton
              onClick={handleClick}
              sx={{
                top: '5rem',
                color: 'grey.100',
                right: '2.5625rem',
                position: 'absolute',
              }}
            >
              <Icon icon="bx:copy" fontSize={20} />
            </IconButton>
          </Tooltip>
          <div>{renderCode()}</div>
        </CardContent>
      </Collapse>
    </Card>
  )
}

export default CardSnippet
