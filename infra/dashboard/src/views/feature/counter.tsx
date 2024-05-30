// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useState } from 'react'
import Card from '@mui/material/Card'
import Button from '@mui/material/Button'
import Typography from '@mui/material/Typography'
import CardContent from '@mui/material/CardContent'
import Grid from '@mui/material/Grid'

const CounterView = () => {
  const [count, setCount] = useState<number>(0)

  const handleAddClick = () => {
    setCount((val) => {
      return val + 1
    })
  }

  return (
    <Card sx={{ position: 'relative' }}>
      <CardContent sx={{ py: (theme) => `${theme.spacing(5)} !important` }}>
        <Grid container spacing={6}>
          <Grid item xs={12} sm={12} sx={{ textAlign: ['center'] }}>
            <Typography variant="h5" sx={{ mb: 4, color: 'primary.main' }}>
              Counter: {count}
            </Typography>
            <Typography sx={{ color: 'text.secondary' }}>
              <Button variant="contained" onClick={handleAddClick}>
                Increase
              </Button>
            </Typography>
          </Grid>
        </Grid>
      </CardContent>
    </Card>
  )
}

export default CounterView
