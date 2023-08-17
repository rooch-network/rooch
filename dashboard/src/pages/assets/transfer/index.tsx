// ** MUI Imports
import Grid from '@mui/material/Grid'

// ** Views Imports
import FeatureView from 'src/views/feature/feature'

// ** Styled Component Import
import ApexChartWrapper from 'src/@core/styles/libs/react-apexcharts'

const SessionPage = () => {
  return (
    <ApexChartWrapper>
      <Grid container spacing={6}>
        <Grid item xs={12} lg={12} sx={{ order: -1 }}>
              <FeatureView/>
        </Grid>
      </Grid>
    </ApexChartWrapper>
  )
}

export default SessionPage
