// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import Grid from '@mui/material/Grid'

// ** Views Imports
import CounterView from 'src/views/feature/counter'

// ** Styled Component Import
import ApexChartWrapper from 'src/@core/styles/libs/react-apexcharts'

const CounterPage = () => {
  return (
    <ApexChartWrapper>
      <Grid container spacing={6}>
        <Grid item xs={12} lg={12} sx={{ order: -1 }}>
          <CounterView />
        </Grid>
      </Grid>
    </ApexChartWrapper>
  )
}

export default CounterPage
