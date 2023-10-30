// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import Grid from '@mui/material/Grid'

// ** Styled Component Import
import Card from '@mui/material/Card'
import CardHeader from '@mui/material/CardHeader'
import CardContent from '@mui/material/CardContent'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { Button } from '@mui/material'
import { useRooch } from '../../../hooks/useRooch'
import { useSession } from '../../../hooks/useSessionAccount'
import { useEffect, useState } from 'react'
import { DevChain } from '@rooch/sdk'

const devCounterAddress = '0x49ee3cf17a017b331ab2b8a4d40ecc9706f328562f9db63cba625a9c106cdf35'
const devCounterModule = `${devCounterAddress}::counter`

const CounterPage = () => {
  const rooch = useRooch()
  const session = useSession()

  const [value, setValue] = useState<number>(0)
  const [loading, setLoading] = useState(true)

  const active = () => {
    return rooch.getActiveChina() === DevChain
  }

  useEffect(() => {
    const fetchCounterValue = async () => {
      const result = await rooch.provider?.executeViewFunction(`${devCounterModule}::value`)

      if (result?.return_values) {
        setValue(parseInt(String(result.return_values[0].decoded_value)))
      }
    }

    fetchCounterValue().finally(() => setLoading(false))
  }, [rooch])

  const handlerIncrease = async () => {
    if (loading) {
      return
    }

    setLoading(true)

    const func = `${devCounterModule}::increase`

    if (session) {
      const result = await session.account?.runFunction(func, [], [], { maxGasAmount: 10000 })
      if (result) {
        setValue(value + 1)
      }
    }

    setLoading(false)
  }

  return (
    <Grid container spacing={6}>
      <Grid item xs={12}>
        <Card>
          <CardHeader title="Counter Example" />
          {
            <CardContent>
              <TextField
                id="address"
                label="Counter Address"
                disabled={true}
                fullWidth
                value={active() ? devCounterAddress : 'This feature is only enabled on dev network'}
              />
            </CardContent>
          }
        </Card>
      </Grid>

      {active() ? (
        <>
          <Grid item xs={12} lg={6}>
            <Card>
              <CardContent>
                <Typography sx={{ fontWeight: 1000, mb: 1, textAlign: 'center' }}>
                  {loading ? 'loading...' : value}
                </Typography>
                <Typography variant="body2" sx={{ my: 'auto', textAlign: 'center' }}>
                  Dev network total counter
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          <Grid item xs={12} lg={6}>
            <Card>
              <CardContent sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center' }}>
                <Button onClick={handlerIncrease} disabled={loading}>
                  Increase
                </Button>
              </CardContent>
            </Card>
          </Grid>
        </>
      ) : (
        <></>
      )}
    </Grid>
  )
}

export default CounterPage
