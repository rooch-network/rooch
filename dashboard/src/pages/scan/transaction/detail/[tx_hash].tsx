// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** MUI Imports
import Card from '@mui/material/Card'
import Table from '@mui/material/Table'
import Divider from '@mui/material/Divider'
import TableRow from '@mui/material/TableRow'
import TableBody from '@mui/material/TableBody'
import Typography from '@mui/material/Typography'
import Box from '@mui/material/Box'
import CardContent from '@mui/material/CardContent'
import { styled, useTheme } from '@mui/material/styles'
import TableContainer from '@mui/material/TableContainer'
import TableCell, { TableCellBaseProps } from '@mui/material/TableCell'

// ** Configs
import { useRouter } from 'next/router'

// ** Store
import { useAppSelector } from 'src/store'

// ** SDK
import { TransactionResultView } from '@rooch/sdk'

const MUITableCell = styled(TableCell)<TableCellBaseProps>(({ theme }) => ({
  borderBottom: 0,
  paddingLeft: '0 !important',
  paddingRight: '0 !important',
  paddingTop: `${theme.spacing(1)} !important`,
  paddingBottom: `${theme.spacing(2)} !important`,
}))

const TransactionDetail = () => {
  // ** Hook
  const theme = useTheme()

  const { result, status } = useAppSelector((state) => state.transaction)

  const router = useRouter()

  const txHash = router.query.tx_hash

  let data: TransactionResultView | undefined

  if (status === 'finished') {
    data = result.data.find((v) => v.execution_info.tx_hash === txHash)
  }

  // TODO:
  if (!data) {
    router.push('/scan/transaction/list')
  }

  if (data) {
    return (
      <Card>
        <CardContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', mb: { sm: 0, xs: 6 } }}>
            <Box sx={{ display: 'flex', alignItems: 'center' }}>
              <span style={{ whiteSpace: 'nowrap' }}>
                <svg
                  width="22.365"
                  height="42.84"
                  viewBox="0 0 71 136"
                  fill="none"
                  xmlns="http://www.w3.org/2000/svg"
                >
                  <path
                    fill={theme.palette.primary.main}
                    d="M0.118329 12.35L0.118328 31.2282L33.2121 61.233H0.260287V74.8389H33.1326L0.118164 104.772V123.619L28.3398 98.4688L28.3452 136H41.7219L41.7461 98.4688L69.949 123.602V104.772L36.9346 74.8389H70.0911V61.233H36.8554L69.9491 31.2282L69.9492 12.3665L41.7461 37.5L41.7221 0H28.3454L28.3398 37.5L0.118329 12.35Z"
                  />
                </svg>
              </span>
              <Typography
                sx={{
                  ml: 2,
                  lineHeight: 1,
                  fontWeight: 700,
                  letterSpacing: '-0.45px',
                  fontSize: '1.75rem !important',
                  whiteSpace: 'nowrap',
                }}
              >
                TRANSACTION HASH:
                <Typography
                  sx={{
                    display: 'inline-flex',
                    ml: 2,
                    fontWeight: 400,
                    whiteSpace: 'nowrap',
                    cursor: 'pointer',
                    fontSize: '1.3rem !important',
                  }}
                >
                  {txHash}
                </Typography>
              </Typography>
            </Box>
          </Box>
        </CardContent>

        <Divider sx={{ my: '0 !important' }} />

        <CardContent>
          <TableContainer>
            <Table>
              <TableBody>
                <TableRow>Sequence Info</TableRow>
                <TableRow>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    Order:
                  </MUITableCell>
                  <MUITableCell sx={{ pb: '0 !important' }}>
                    {data.sequence_info.tx_order}
                  </MUITableCell>
                </TableRow>
                <TableRow>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    Auth Validator ID:
                  </MUITableCell>
                  <MUITableCell sx={{ pb: '0 !important' }}>
                    {data.sequence_info.tx_order_signature.auth_validator_id}
                  </MUITableCell>
                </TableRow>
                <TableRow>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    Accumulator Root:
                  </MUITableCell>
                  <MUITableCell sx={{ pb: '0 !important' }}>
                    {data.sequence_info.tx_accumulator_root}
                  </MUITableCell>
                </TableRow>
                <TableRow>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    Signature Payload:{' '}
                  </MUITableCell>
                  <MUITableCell sx={{ pb: '0 !important' }}>
                    {data.sequence_info.tx_order_signature.payload}
                  </MUITableCell>
                </TableRow>
                <TableRow>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    <Divider sx={{ mt: 2, mb: 2 }} />{' '}
                  </MUITableCell>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    <Divider sx={{ mt: 2, mb: 2 }} />
                  </MUITableCell>
                </TableRow>
                <TableRow>Transaction Info</TableRow>
                <TableRow>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    Sender:
                  </MUITableCell>
                  <MUITableCell sx={{ pb: '0 !important' }}>{data.transaction.sender}</MUITableCell>
                </TableRow>
                <TableRow>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    Action Type:
                  </MUITableCell>
                  <MUITableCell sx={{ pb: '0 !important', textTransform: 'uppercase' }}>
                    {data.transaction.action_type}
                  </MUITableCell>
                </TableRow>
                <TableRow>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    Transaction Type:
                  </MUITableCell>
                  <MUITableCell sx={{ pb: '0 !important', textTransform: 'uppercase' }}>
                    {data.transaction.transaction_type}
                  </MUITableCell>
                </TableRow>
                <TableRow>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    <Divider sx={{ mt: 2, mb: 2 }} />{' '}
                  </MUITableCell>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    <Divider sx={{ mt: 2, mb: 2 }} />
                  </MUITableCell>
                </TableRow>
                <TableRow>Execution Info</TableRow>
                <TableRow>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    Status:
                  </MUITableCell>
                  <MUITableCell sx={{ pb: '0 !important', textTransform: 'uppercase' }}>
                    {data.execution_info.status.type}
                  </MUITableCell>
                </TableRow>
                <TableRow>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    Gas Used:
                  </MUITableCell>
                  <MUITableCell sx={{ pb: '0 !important', textTransform: 'uppercase' }}>
                    {data.execution_info.gas_used}
                  </MUITableCell>
                </TableRow>
                <TableRow>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    Event Root:
                  </MUITableCell>
                  <MUITableCell sx={{ pb: '0 !important', textTransform: 'uppercase' }}>
                    {data.execution_info.event_root}
                  </MUITableCell>
                </TableRow>
                <TableRow>
                  <MUITableCell sx={{ pb: '0 !important', whiteSpace: 'nowrap' }}>
                    State Root:
                  </MUITableCell>
                  <MUITableCell sx={{ pb: '0 !important', textTransform: 'uppercase' }}>
                    {data.execution_info.state_root}
                  </MUITableCell>
                </TableRow>
              </TableBody>
            </Table>
          </TableContainer>
        </CardContent>
      </Card>
    )
  } else {
    return null
  }
}

export default TransactionDetail
