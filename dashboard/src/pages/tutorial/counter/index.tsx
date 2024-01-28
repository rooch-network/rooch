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
import React, { useState } from 'react'
import { DevNetwork } from '@roochnetwork/rooch-sdk'
import {
  useCurrentSessionAccount,
  useRoochClient,
  useRoochClientQuery,
} from '@roochnetwork/rooch-sdk-kit'
import SessionGuard from 'src/auth/sessionGuard'

const devCounterAddress = '0x49ee3cf17a017b331ab2b8a4d40ecc9706f328562f9db63cba625a9c106cdf35'
const devCounterModule = `${devCounterAddress}::counter`

const CounterPage = () => {
  const [loading, setLoading] = useState(false)
  const client = useRoochClient()
  const active = () => {
    return client.network === DevNetwork
  }

  let { data, isPending, refetch } = useRoochClientQuery('executeViewFunction', {
    funcId: `${devCounterModule}::value`,
  })

  const sessionKey = useCurrentSessionAccount()

  const handlerIncrease = () => {
    // if (sessionKey === null) {
    //   setAuth(true)
    //
    //   return
    // }

    if (loading) {
      return
    }

    setLoading(true)

    const func = `${devCounterModule}::increase`

    sessionKey?.runFunction(func, [], [], { maxGasAmount: 1000000 }).finally(() => {
      setLoading(false)
      refetch()
    })
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
        <SessionGuard>
          <>
            <Grid item xs={12} lg={6}>
              <Card
                sx={{
                  height: '100%',
                  display: 'flex',
                  flexDirection: 'column',
                  justifyContent: 'center',
                  alignItems: 'center',
                }}
              >
                <CardContent>
                  <Typography sx={{ fontWeight: 1000, mb: 1, textAlign: 'center' }}>
                    {isPending ? 'loading...' : (data!.return_values![0].decoded_value as string)}
                  </Typography>
                  <Typography variant="body2" sx={{ my: 'auto', textAlign: 'center' }}>
                    Dev network total counter
                  </Typography>
                </CardContent>
              </Card>
            </Grid>
            <Grid item xs={12} lg={6}>
              <Card
                sx={{
                  height: '100%',
                  display: 'flex',
                  flexDirection: 'column',
                  justifyContent: 'center',
                  alignItems: 'center',
                }}
              >
                <CardContent
                  sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center' }}
                >
                  <Button onClick={handlerIncrease} disabled={loading}>
                    Increase
                  </Button>
                </CardContent>
              </Card>
            </Grid>
          </>
        </SessionGuard>
      ) : (
        <></>
      )}
    </Grid>
  )
}

export default CounterPage
