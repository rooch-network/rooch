// ** React Imports
import { MouseEvent, useState } from 'react'

// ** MUI Imports
import Box from '@mui/material/Box'
import Card from '@mui/material/Card'
import Menu from '@mui/material/Menu'
import Button from '@mui/material/Button'
import MenuItem from '@mui/material/MenuItem'
import Typography from '@mui/material/Typography'
import CardContent from '@mui/material/CardContent'

// ** Types Imports
import { ThemeColor } from 'src/@core/layouts/types'
import { CardStatsTargetProps } from 'src/@core/components/card-statistics/types'

// ** Icons Imports
import Icon from 'src/@core/components/icon'

// ** Custom Component Import
import CustomAvatar from 'src/@core/components/mui/avatar'

// ** Hook Import
import { useSettings } from 'src/@core/hooks/useSettings'

//  ** Util Import
import { hexToRGBA } from 'src/@core/utils/hex-to-rgba'

const CardStatsTarget = (props: CardStatsTargetProps) => {
  // ** Props
  const {
    title,
    stats,
    subtitle,
    avatarIcon,
    buttonText,
    trendNumber,
    buttonOptions,
    avatarIconProps,
    trend = 'positive',
    avatarColor = 'primary'
  } = props

  // ** State
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null)

  // ** Hook & Var
  const { settings } = useSettings()
  const { direction } = settings

  const handleClick = (event: MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget)
  }

  const handleClose = () => {
    setAnchorEl(null)
  }

  return (
    <Card sx={{ overflow: 'visible', position: 'relative' }}>
      <CardContent sx={{ pt: 4, px: 4 }}>
        <Box sx={{ mb: 6, display: 'flex', justifyContent: 'space-between' }}>
          <Typography sx={{ mr: 2, fontWeight: 500 }}>{title}</Typography>
          <Button
            size='small'
            color='secondary'
            aria-haspopup='true'
            onClick={handleClick}
            sx={{ p: theme => theme.spacing(0, 2), '&:hover': { transform: 'none !important' } }}
          >
            <Typography variant='body2' sx={{ mr: 1, textTransform: 'none' }}>
              {buttonText}
            </Typography>
            <Icon fontSize={16} icon='bx:chevron-down' />
          </Button>
          <Menu
            keepMounted
            anchorEl={anchorEl}
            onClose={handleClose}
            open={Boolean(anchorEl)}
            anchorOrigin={{ vertical: 'bottom', horizontal: direction === 'ltr' ? 'right' : 'left' }}
            transformOrigin={{ vertical: 'top', horizontal: direction === 'ltr' ? 'right' : 'left' }}
          >
            {buttonOptions.map((option: string, index: number) => (
              <MenuItem key={index} onClick={handleClose}>
                {option}
              </MenuItem>
            ))}
          </Menu>
        </Box>
        <Box sx={{ display: 'flex', alignItems: 'center', flexDirection: 'column' }}>
          <CustomAvatar
            skin='light'
            color={avatarColor}
            sx={{
              mb: 4.5,
              width: 42,
              height: 42,
              boxShadow: theme => `0 0 0 5px ${hexToRGBA(theme.palette[avatarColor as ThemeColor].main, 0.04)}`
            }}
          >
            <Icon icon={avatarIcon} {...avatarIconProps} />
          </CustomAvatar>
          <Typography variant='h5'>{stats}</Typography>
          <Typography variant='body2' sx={{ mb: 2 }}>
            {subtitle}
          </Typography>
          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              '& svg': { color: `${trend === 'positive' ? 'success' : 'error'}.main` }
            }}
          >
            <Typography sx={{ mr: 0.5, color: `${trend === 'positive' ? 'success' : 'error'}.main` }}>
              {`${trendNumber}%`}
            </Typography>
            <Icon fontSize={20} icon={trend === 'positive' ? 'bx:chevron-up' : 'bx:chevron-down'} />
          </Box>
        </Box>
      </CardContent>
    </Card>
  )
}

export default CardStatsTarget
