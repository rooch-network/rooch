// ** React Imports
import { MouseEvent, useState } from 'react'

// ** MUI Import
import Box from '@mui/material/Box'
import Card from '@mui/material/Card'
import Menu from '@mui/material/Menu'
import Button from '@mui/material/Button'
import MenuItem from '@mui/material/MenuItem'
import Typography from '@mui/material/Typography'
import CardContent from '@mui/material/CardContent'
import Grid, { GridProps } from '@mui/material/Grid'
import { styled, useTheme } from '@mui/material/styles'

// ** Icons Imports
import Icon from 'src/@core/components/icon'

// ** Third Party Imports
import { ApexOptions } from 'apexcharts'

// ** Custom Components Imports
import CustomAvatar from 'src/@core/components/mui/avatar'
import ReactApexcharts from 'src/@core/components/react-apexcharts'

// ** Hook Import
import { useSettings } from 'src/@core/hooks/useSettings'

// ** Util Import
import { hexToRGBA } from 'src/@core/utils/hex-to-rgba'

const tokenOptions = ['BTC', 'ROH', 'APT']

const series = [
  { name: `${new Date().getFullYear() - 1}`, data: [18, 7, 15, 29, 18, 12, 9] },
  { name: `${new Date().getFullYear() - 2}`, data: [-13, -18, -9, -14, -5, -17, -15] }
]

const StyledGrid = styled(Grid)<GridProps>(({ theme }) => ({
  [theme.breakpoints.down('sm')]: {
    borderBottom: `1px solid ${theme.palette.divider}`
  },
  [theme.breakpoints.up('sm')]: {
    borderRight: `1px solid ${theme.palette.divider}`
  }
}))

const AnalyticsTotalRevenue = () => {
  // ** State
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null)

  // ** Hooks & Var
  const theme = useTheme()
  const { settings } = useSettings()
  const { direction } = settings

  const handleClick = (event: MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget)
  }

  const handleClose = () => {
    setAnchorEl(null)
  }

  const barOptions: ApexOptions = {
    chart: {
      stacked: true,
      parentHeightOffset: 0,
      toolbar: { show: false }
    },
    dataLabels: { enabled: false },
    stroke: {
      width: 6,
      lineCap: 'round',
      colors: [theme.palette.background.paper]
    },
    colors: [hexToRGBA(theme.palette.primary.main, 1), hexToRGBA(theme.palette.info.main, 1)],
    legend: {
      offsetX: -10,
      position: 'top',
      fontSize: '14px',
      horizontalAlign: 'left',
      fontFamily: theme.typography.fontFamily,
      labels: {
        colors: theme.palette.text.secondary
      },
      itemMargin: {
        vertical: 4,
        horizontal: 10
      },
      markers: {
        width: 8,
        height: 8,
        radius: 10,
        offsetX: -4
      }
    },
    states: {
      hover: {
        filter: { type: 'none' }
      },
      active: {
        filter: { type: 'none' }
      }
    },
    grid: {
      borderColor: theme.palette.divider,
      padding: {
        bottom: 5
      }
    },
    plotOptions: {
      bar: {
        borderRadius: 10,
        columnWidth: '30%',
        endingShape: 'rounded',
        startingShape: 'rounded'
      }
    },
    xaxis: {
      axisTicks: { show: false },
      crosshairs: { opacity: 0 },
      axisBorder: { show: false },
      categories: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul'],
      labels: {
        style: {
          fontSize: '14px',
          colors: theme.palette.text.disabled,
          fontFamily: theme.typography.fontFamily
        }
      }
    },
    yaxis: {
      labels: {
        style: {
          fontSize: '14px',
          colors: theme.palette.text.disabled,
          fontFamily: theme.typography.fontFamily
        }
      }
    },
    responsive: [
      {
        breakpoint: theme.breakpoints.values.xl,
        options: {
          plotOptions: {
            bar: { columnWidth: '43%' }
          }
        }
      },
      {
        breakpoint: theme.breakpoints.values.lg,
        options: {
          plotOptions: {
            bar: { columnWidth: '30%' }
          }
        }
      },
      {
        breakpoint: theme.breakpoints.values.md,
        options: {
          plotOptions: {
            bar: { columnWidth: '42%' }
          }
        }
      },
      {
        breakpoint: theme.breakpoints.values.sm,
        options: {
          plotOptions: {
            bar: { columnWidth: '45%' }
          }
        }
      }
    ]
  }

  const radialBarOptions: ApexOptions = {
    chart: {
      sparkline: { enabled: true }
    },
    labels: ['Growth'],
    stroke: { dashArray: 5 },
    colors: [hexToRGBA(theme.palette.primary.main, 1)],
    states: {
      hover: {
        filter: { type: 'none' }
      },
      active: {
        filter: { type: 'none' }
      }
    },
    fill: {
      type: 'gradient',
      gradient: {
        shade: 'dark',
        opacityTo: 0.6,
        opacityFrom: 1,
        shadeIntensity: 0.5,
        stops: [30, 70, 100],
        inverseColors: false,
        gradientToColors: [theme.palette.primary.main]
      }
    },
    plotOptions: {
      radialBar: {
        endAngle: 150,
        startAngle: -140,
        hollow: { size: '55%' },
        track: { background: 'transparent' },
        dataLabels: {
          name: {
            offsetY: 25,
            fontWeight: 600,
            fontSize: '16px',
            color: theme.palette.text.secondary,
            fontFamily: theme.typography.fontFamily
          },
          value: {
            offsetY: -15,
            fontWeight: 500,
            fontSize: '24px',
            color: theme.palette.text.primary,
            fontFamily: theme.typography.fontFamily
          }
        }
      }
    },
    responsive: [
      {
        breakpoint: 900,
        options: {
          chart: { height: 200 }
        }
      },
      {
        breakpoint: 735,
        options: {
          chart: { height: 200 }
        }
      },
      {
        breakpoint: 660,
        options: {
          chart: { height: 200 }
        }
      },
      {
        breakpoint: 600,
        options: {
          chart: { height: 280 }
        }
      }
    ]
  }

  return (
    <Card>
      <Grid container>
        <StyledGrid
          item
          sm={7}
          xl={8}
          xs={12}
          sx={{ '& .apexcharts-series[rel="2"]': { transform: 'translateY(-10px)' } }}
        >
          <CardContent sx={{ p: `${theme.spacing(5, 6, 0)} !important` }}>
            <Typography variant='h6'>Total Assets</Typography>
          </CardContent>
          <ReactApexcharts type='bar' height={312} options={barOptions} series={series} />
        </StyledGrid>
        <Grid item xs={12} sm={5} xl={4}>
          <CardContent sx={{ p: `${theme.spacing(8, 6, 7.5)} !important` }}>
            <Box sx={{ textAlign: 'center' }}>
              <Button
                size='small'
                variant='outlined'
                aria-haspopup='true'
                onClick={handleClick}
                sx={{ '& svg': { ml: 0.5 } }}
              >
                ALL
                <Icon icon='bx:chevron-down' />
              </Button>
              <Menu
                keepMounted
                anchorEl={anchorEl}
                onClose={handleClose}
                open={Boolean(anchorEl)}
                anchorOrigin={{ vertical: 'bottom', horizontal: direction === 'ltr' ? 'right' : 'left' }}
                transformOrigin={{ vertical: 'top', horizontal: direction === 'ltr' ? 'right' : 'left' }}
              >
                {tokenOptions.map((token: string) => (
                  <MenuItem key={token} onClick={handleClose}>
                    {token}
                  </MenuItem>
                ))}
              </Menu>
              <ReactApexcharts type='radialBar' height={200} series={[78]} options={radialBarOptions} />
              <Typography sx={{ mb: 7.5, fontWeight: 600, color: 'text.secondary' }}>62% Assets Growth</Typography>
            </Box>
            <Box sx={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'center' }}>
              <Box sx={{ mr: 4, display: 'flex', alignItems: 'center' }}>
                <CustomAvatar skin='light' variant='rounded' sx={{ mr: 2.5, width: 38, height: 38 }}>
                  <Icon icon='bx:dollar' />
                </CustomAvatar>
                <Box sx={{ display: 'flex', flexDirection: 'column' }}>
                  <Typography variant='body2'>{new Date().getFullYear()}</Typography>
                  <Typography sx={{ fontWeight: 500 }}>$32.5k</Typography>
                </Box>
              </Box>
              <Box sx={{ display: 'flex', alignItems: 'center' }}>
                <CustomAvatar skin='light' color='info' variant='rounded' sx={{ mr: 2.5, width: 38, height: 38 }}>
                  <Icon icon='bx:wallet' />
                </CustomAvatar>
                <Box sx={{ display: 'flex', flexDirection: 'column' }}>
                  <Typography variant='body2'>{new Date().getFullYear() - 1}</Typography>
                  <Typography sx={{ fontWeight: 500 }}>$41.2k</Typography>
                </Box>
              </Box>
            </Box>
          </CardContent>
        </Grid>
      </Grid>
    </Card>
  )
}

export default AnalyticsTotalRevenue
