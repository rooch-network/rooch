// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import Grid from '@mui/material/Grid'

// ** Views Imports
import SessionKeyList from 'src/views/feature/SessionKeyList'

// ** Styled Component Import
import ApexChartWrapper from 'src/@core/styles/libs/react-apexcharts'

const SessionPage = () => {
  return (
    <ApexChartWrapper>
      <Grid container spacing={6}>
        <Grid item xs={12} lg={12} sx={{ order: -1 }}>
          <SessionKeyList />
        </Grid>
      </Grid>
    </ApexChartWrapper>
  )
}

export default SessionPage
