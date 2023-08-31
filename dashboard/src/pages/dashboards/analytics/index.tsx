// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import Grid from '@mui/material/Grid'

// ** Demo Component Imports
import AnalyticsTotalRevenue from 'src/views/dashboards/analytics/AnalyticsTotalRevenue'
import AnalyticsTransactions from 'src/views/dashboards/analytics/AnalyticsTransactions'
import AnalyticsCongratulations from 'src/views/dashboards/analytics/AnalyticsCongratulations'

// ** Styled Component Import
import ApexChartWrapper from 'src/@core/styles/libs/react-apexcharts'

const AnalyticsDashboard = () => {
  return (
    <ApexChartWrapper>
      <Grid container spacing={6}>
        <Grid item xs={12} lg={8} sx={{ order: -1 }}>
          <Grid container spacing={6}>
            <Grid item xs={12}>
              <AnalyticsCongratulations />
            </Grid>
            <Grid item xs={12}>
              <AnalyticsTotalRevenue />
            </Grid>
          </Grid>
        </Grid>
        <Grid item xs={12} md={12} lg={4} sx={{ order: -1 }}>
          <AnalyticsTransactions />
        </Grid>
      </Grid>
    </ApexChartWrapper>
  )
}

export default AnalyticsDashboard
