// ** MUI Imports
import Box from '@mui/material/Box'
import Card from '@mui/material/Card'
import Avatar from '@mui/material/Avatar'
import Typography from '@mui/material/Typography'
import CardHeader from '@mui/material/CardHeader'
import CardContent from '@mui/material/CardContent'

// ** Custom Components Import
import OptionsMenu from 'src/@core/components/option-menu'

interface DataType {
  title: string
  imgSrc: string
  amount: string
  subtitle: string
}

const data: DataType[] = [
  {
    title: 'Withdraw',
    amount: '-82.6',
    subtitle: '0x123456...',
    imgSrc: '/images/cards/credit-card.png'
  },
  {
    title: 'Deposit',
    amount: '+270.69',
    subtitle: "0x123456...",
    imgSrc: '/images/cards/atm-card.png'
  },
  {
    title: 'Transfer',
    amount: '+637.91',
    subtitle: '0x123456...',
    imgSrc: '/images/cards/paypal.png'
  },
  {
    title: 'Withdraw',
    amount: '-82.6',
    subtitle: '0x123456...',
    imgSrc: '/images/cards/credit-card.png'
  },
  {
    title: 'Deposit',
    amount: '+270.69',
    subtitle: "0x123456...",
    imgSrc: '/images/cards/atm-card.png'
  },
  {
    title: 'Transfer',
    amount: '+637.91',
    subtitle: '0x123456...',
    imgSrc: '/images/cards/paypal.png'
  },
  {
    title: 'Transfer',
    amount: '+637.91',
    subtitle: '0x123456...',
    imgSrc: '/images/cards/paypal.png'
  },
]

const AnalyticsTransactions = () => {
  return (
    <Card>
      <CardHeader
        title='Transactions'
        action={<OptionsMenu iconButtonProps={{ size: 'small' }} options={['Share', 'Refresh']} />}
      />
      <CardContent>
        {data.map((item: DataType, index: number) => {
          return (
            <Box
              key={index}
              sx={{
                display: 'flex',
                alignItems: 'center',
                mb: index !== data.length - 1 ? 5 : undefined
              }}
            >
              <Avatar src={item.imgSrc} variant='rounded' sx={{ mr: 3.5, width: 38, height: 38 }} />
              <Box
                sx={{
                  width: '100%',
                  display: 'flex',
                  flexWrap: 'wrap',
                  alignItems: 'center',
                  justifyContent: 'space-between'
                }}
              >
                <Box sx={{ mr: 2, display: 'flex', flexDirection: 'column' }}>
                  <Typography variant='body2' sx={{ mb: 0.5, color: 'text.disabled' }}>
                    {item.title}
                  </Typography>
                  <Typography sx={{ fontWeight: 500 }}>{item.subtitle}</Typography>
                </Box>
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                  <Typography sx={{ mr: 3, fontWeight: 500 }}>{item.amount}</Typography>
                  <Typography sx={{ color: 'text.disabled' }}>USD</Typography>
                </Box>
              </Box>
            </Box>
          )
        })}
      </CardContent>
    </Card>
  )
}

export default AnalyticsTransactions
