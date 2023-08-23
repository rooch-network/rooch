// ** MUI Imports
import Box from '@mui/material/Box'
import Card from '@mui/material/Card'
import Typography from '@mui/material/Typography'
import CardContent from '@mui/material/CardContent'

// ** Types Imports
import { CardStatsHorizontalProps } from 'src/@core/components/card-statistics/types'

// ** Icon Import
import Icon from 'src/@core/components/icon'

// ** Custom Component Imports
import CustomAvatar from 'src/@core/components/mui/avatar'

const CardStatsHorizontal = (props: CardStatsHorizontalProps) => {
  // ** Props
  const {
    title,
    stats,
    subtitle,
    avatarIcon,
    trendNumber,
    avatarIconProps,
    trend = 'positive',
    avatarColor = 'primary'
  } = props

  return (
    <Card>
      <CardContent>
        <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
          <Box sx={{ display: 'flex', flexDirection: 'column' }}>
            <Typography sx={{ mb: 1.75, color: 'text.secondary' }}>{title}</Typography>
            <Box sx={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center' }}>
              <Typography variant='h5' sx={{ mr: 2.5 }}>
                {stats}
              </Typography>
              <Typography variant='body2' sx={{ mt: 1, color: `${trend === 'negative' ? 'error' : 'success'}.main` }}>
                {`(${trend === 'negative' ? '-' : '+'}${trendNumber}%)`}
              </Typography>
            </Box>
            <Typography variant='body2'>{subtitle}</Typography>
          </Box>
          <CustomAvatar skin='light' variant='rounded' color={avatarColor} sx={{ width: 42, height: 42 }}>
            <Icon icon={avatarIcon} {...avatarIconProps} />
          </CustomAvatar>
        </Box>
      </CardContent>
    </Card>
  )
}

export default CardStatsHorizontal
