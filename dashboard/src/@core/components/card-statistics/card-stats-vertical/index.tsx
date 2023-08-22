// ** MUI Imports
import Box from '@mui/material/Box'
import Card from '@mui/material/Card'
import Typography from '@mui/material/Typography'
import CardContent from '@mui/material/CardContent'

// ** Types Imports
import { CardStatsVerticalProps } from 'src/@core/components/card-statistics/types'

// ** Icon Import
import Icon from 'src/@core/components/icon'

// ** Custom Components Imports
import CustomAvatar from 'src/@core/components/mui/avatar'
import OptionsMenu from 'src/@core/components/option-menu'

const CardStatsVertical = (props: CardStatsVerticalProps) => {
  // ** Props
  const {
    title,
    stats,
    avatarSrc,
    avatarIcon,
    trendNumber,
    optionsMenuProps,
    trend = 'positive',
    avatarColor = 'primary'
  } = props

  return (
    <Card>
      <CardContent sx={{ p: theme => `${theme.spacing(5, 5, 4)} !important` }}>
        <Box sx={{ display: 'flex', mb: 4, alignItems: 'flex-start', justifyContent: 'space-between' }}>
          <CustomAvatar
            skin='light'
            variant='rounded'
            color={avatarColor}
            src={avatarSrc ?? ''}
            sx={{ width: 42, height: 42 }}
          >
            {avatarIcon && !avatarSrc ? avatarIcon : null}
          </CustomAvatar>
          {optionsMenuProps ? (
            <OptionsMenu {...optionsMenuProps} />
          ) : (
            <OptionsMenu
              options={['Refresh', 'Share', 'Update']}
              iconButtonProps={{ size: 'small', className: 'card-more-options', sx: { color: 'text.secondary' } }}
            />
          )}
        </Box>
        <Typography sx={{ mb: 0.5, fontWeight: 600, color: 'text.secondary' }}>{title}</Typography>
        <Typography variant='h5' sx={{ mb: 2 }}>
          {stats}
        </Typography>
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            '& svg': { mr: 1, color: `${trend === 'positive' ? 'success' : 'error'}.main` }
          }}
        >
          <Icon fontSize={16} icon={trend === 'positive' ? 'bx:up-arrow-alt' : 'bx:down-arrow-alt'} />
          <Typography
            variant='body2'
            sx={{ fontWeight: 500, color: `${trend === 'positive' ? 'success' : 'error'}.main` }}
          >
            {`${trendNumber}%`}
          </Typography>
        </Box>
      </CardContent>
    </Card>
  )
}

export default CardStatsVertical
