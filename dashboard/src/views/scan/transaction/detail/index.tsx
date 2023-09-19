// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Next Import
import { GetStaticPaths, GetStaticProps, GetStaticPropsContext, InferGetStaticPropsType } from 'next/types'

// ** MUI Imports
import Grid from '@mui/material/Grid'
import Card from '@mui/material/Card'
import Table from '@mui/material/Table'
import Divider from '@mui/material/Divider'
import TableRow from '@mui/material/TableRow'
import TableHead from '@mui/material/TableHead'
import TableBody from '@mui/material/TableBody'
import Typography from '@mui/material/Typography'
import Box, { BoxProps } from '@mui/material/Box'
import CardContent from '@mui/material/CardContent'
import { styled, useTheme } from '@mui/material/styles'
import TableContainer from '@mui/material/TableContainer'
import TableCell, { TableCellBaseProps } from '@mui/material/TableCell'

// ** Configs
import themeConfig from 'src/configs/themeConfig'
import {useAppDispatch, useAppSelector} from "../../../../store";

interface Props {
    txHash: string | undefined
}

const MUITableCell = styled(TableCell)<TableCellBaseProps>(({ theme }) => ({
    borderBottom: 0,
    paddingLeft: '0 !important',
    paddingRight: '0 !important',
    paddingTop: `${theme.spacing(1)} !important`,
    paddingBottom: `${theme.spacing(2)} !important`
}))

const CalcWrapper = styled(Box)<BoxProps>(({ theme }) => ({
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    '&:not(:last-of-type)': {
        marginBottom: theme.spacing(2)
    }
}))

const PreviewCard = ({ txHash }: Props) => {
    // ** Hook
    const theme = useTheme()

    const {result, status, error} = useAppSelector((state) => state.transaction)

    console.log(txHash)
    console.log(result)

    return (<>hahah</>)
    // if (data) {
    //     return (
    //         <Card>
    //             <CardContent>
    //                 <Grid container sx={{ p: { sm: 4, xs: 0 } }}>
    //                     <Grid item sm={6} xs={12}>
    //                         <Box sx={{ display: 'flex', flexDirection: 'column', mb: { sm: 0, xs: 6 } }}>
    //                             <Box sx={{ mb: 6, display: 'flex', alignItems: 'center' }}>
    //                                 <svg width={22} height={32} viewBox='0 0 55 81' fill='none' xmlns='http://www.w3.org/2000/svg'>
    //                                     <path
    //                                         fill={theme.palette.primary.main}
    //                                         d='M30.1984 0.0144043C24.8945 0.425781 25.2534 6.16968 26.6435 7.65326C22.693 10.3649 13.1875 16.8867 6.76944 21.2803C1.21531 25.0824 -0.842975 34.6064 1.11159 40.8262C3.00952 46.8658 12.4904 51.3615 17.5337 52.7256C17.5337 52.7256 11.7188 56.0269 6.60358 60.0482C1.48831 64.0695 -0.622615 69.3436 3.06836 75.262C6.75933 81.1805 12.725 80.761 17.5257 78.6229C22.3264 76.4848 32.1683 69.1692 37.9402 65.1633C42.7282 61.5411 43.9669 53.6444 41.7631 46.9643C39.9758 41.5468 30.0969 36.4284 25.1792 34.6064C27.1946 33.1595 32.4935 29.4242 37.129 26.0909C38.7184 30.5636 43.9998 30.212 45.6103 27.8209C47.6216 23.4326 51.8339 13.4663 53.9579 8.55175C54.8862 4.81044 52.5639 2.78457 50.2227 2.35938C46.8672 1.75 38.3222 0.960115 30.1984 0.0144043Z'
    //                                     />
    //                                     <path
    //                                         fillOpacity='0.2'
    //                                         fill={theme.palette.common.white}
    //                                         d='M26.6523 7.65625C24.9492 5.625 25.3239 0.255308 30.2922 0.0105286C33.0074 0.326611 35.7804 0.62685 38.3907 0.909477C43.5904 1.47246 48.1446 1.96556 50.311 2.3748C52.7331 2.83234 54.886 5.06072 53.9543 8.61103C53.2063 10.3418 52.2075 12.6646 51.1482 15.1282C49.1995 19.6601 47.0459 24.6685 45.8717 27.3445C44.7224 29.964 39.111 31.0585 37.1137 26.0951C32.4782 29.4283 27.2884 33.1556 25.273 34.6026C24.931 34.4553 24.3074 34.2381 23.5124 33.9613C20.8691 33.0407 16.331 31.4602 13.9477 29.5966C9.61363 25.5918 11.6259 19.4662 13.1737 16.904C17.8273 13.7183 20.7417 11.7161 23.4984 9.82236C24.5437 9.10427 25.5662 8.40178 26.6523 7.65625Z'
    //                                     />
    //                                     <path
    //                                         fillOpacity='0.2'
    //                                         fill={theme.palette.common.white}
    //                                         d='M17.543 52.7266C21.2241 53.9875 28.5535 57.0509 30.091 59.101C32.0129 61.6635 33.1576 64.34 29.2527 71.2039C28.5954 71.6481 27.9821 72.0633 27.4069 72.4528C22.1953 75.9817 20.1085 77.3946 16.6243 79.0531C13.5855 80.2464 6.61575 81.7103 2.66559 74.5653C-1.11764 67.7222 3.23818 62.7113 6.5963 60.065L12.1695 56.0339L14.8359 54.3477L17.543 52.7266Z'
    //                                     />
    //                                 </svg>
    //                                 <Typography
    //                                     variant='h5'
    //                                     sx={{
    //                                         ml: 2,
    //                                         lineHeight: 1,
    //                                         fontWeight: 700,
    //                                         letterSpacing: '-0.45px',
    //                                         textTransform: 'lowercase',
    //                                         fontSize: '1.75rem !important'
    //                                     }}
    //                                 >
    //                                     {themeConfig.templateName}
    //                                 </Typography>
    //                             </Box>
    //                             <div>
    //                                 <Typography sx={{ mb: 1, color: 'text.secondary' }}>Office 149, 450 South Brand Brooklyn</Typography>
    //                                 <Typography sx={{ mb: 1, color: 'text.secondary' }}>San Diego County, CA 91905, USA</Typography>
    //                                 <Typography sx={{ color: 'text.secondary' }}>+1 (123) 456 7891, +44 (876) 543 2198</Typography>
    //                             </div>
    //                         </Box>
    //                     </Grid>
    //                     <Grid item sm={6} xs={12}>
    //                         <Box sx={{ display: 'flex', justifyContent: { xs: 'flex-start', sm: 'flex-end' } }}>
    //                             <Table sx={{ maxWidth: '200px' }}>
    //                                 <TableBody>
    //                                     <TableRow>
    //                                         <MUITableCell>
    //                                             <Typography variant='h5'>Invoice</Typography>
    //                                         </MUITableCell>
    //                                         <MUITableCell>
    //                                             <Typography variant='h5'>{`#${data.invoice.id}`}</Typography>
    //                                         </MUITableCell>
    //                                     </TableRow>
    //                                     <TableRow>
    //                                         <MUITableCell>
    //                                             <Typography sx={{ color: 'text.secondary' }}>Date Issued:</Typography>
    //                                         </MUITableCell>
    //                                         <MUITableCell>
    //                                             <Typography sx={{ color: 'text.secondary', fontWeight: 600 }}>
    //                                                 {data.invoice.issuedDate}
    //                                             </Typography>
    //                                         </MUITableCell>
    //                                     </TableRow>
    //                                     <TableRow>
    //                                         <MUITableCell>
    //                                             <Typography sx={{ color: 'text.secondary' }}>Date Due:</Typography>
    //                                         </MUITableCell>
    //                                         <MUITableCell>
    //                                             <Typography sx={{ color: 'text.secondary', fontWeight: 600 }}>
    //                                                 {data.invoice.dueDate}
    //                                             </Typography>
    //                                         </MUITableCell>
    //                                     </TableRow>
    //                                 </TableBody>
    //                             </Table>
    //                         </Box>
    //                     </Grid>
    //                 </Grid>
    //             </CardContent>
    //
    //             <Divider sx={{ my: '0 !important' }} />
    //
    //             <CardContent>
    //                 <Grid container sx={{ p: { sm: 4, xs: 0 }, pb: theme => `${theme.spacing(1)} !important` }}>
    //                     <Grid item xs={12} sm={6} sx={{ mb: { lg: 0, xs: 5 } }}>
    //                         <Typography sx={{ mb: 4, fontWeight: 500 }}>Invoice To:</Typography>
    //                         <Typography sx={{ mb: 1, color: 'text.secondary' }}>{data.invoice.name}</Typography>
    //                         <Typography sx={{ mb: 1, color: 'text.secondary' }}>{data.invoice.company}</Typography>
    //                         <Typography sx={{ mb: 1, color: 'text.secondary' }}>{data.invoice.address}</Typography>
    //                         <Typography sx={{ mb: 1, color: 'text.secondary' }}>{data.invoice.contact}</Typography>
    //                         <Typography sx={{ mb: 1, color: 'text.secondary' }}>{data.invoice.companyEmail}</Typography>
    //                     </Grid>
    //                     <Grid
    //                         item
    //                         sm={6}
    //                         xs={12}
    //                         sx={{
    //                             display: 'flex',
    //                             px: { sm: 4, xs: 0 },
    //                             justifyContent: ['flex-start', 'flex-end']
    //                         }}
    //                     >
    //                         <div>
    //                             <Typography sx={{ mb: 4, color: 'text.secondary', fontWeight: 500 }}>Bill To:</Typography>
    //                             <TableContainer>
    //                                 <Table>
    //                                     <TableBody>
    //                                         <TableRow>
    //                                             <MUITableCell sx={{ pb: '0 !important' }}>Total Due:</MUITableCell>
    //                                             <MUITableCell sx={{ pb: '0 !important' }}>{data.paymentDetails.totalDue}</MUITableCell>
    //                                         </TableRow>
    //                                         <TableRow>
    //                                             <MUITableCell sx={{ pb: '0 !important' }}>Bank name:</MUITableCell>
    //                                             <MUITableCell sx={{ pb: '0 !important' }}>{data.paymentDetails.bankName}</MUITableCell>
    //                                         </TableRow>
    //                                         <TableRow>
    //                                             <MUITableCell sx={{ pb: '0 !important' }}>Country:</MUITableCell>
    //                                             <MUITableCell sx={{ pb: '0 !important' }}>{data.paymentDetails.country}</MUITableCell>
    //                                         </TableRow>
    //                                         <TableRow>
    //                                             <MUITableCell sx={{ pb: '0 !important' }}>IBAN:</MUITableCell>
    //                                             <MUITableCell sx={{ pb: '0 !important' }}>{data.paymentDetails.iban}</MUITableCell>
    //                                         </TableRow>
    //                                         <TableRow>
    //                                             <MUITableCell>SWIFT code:</MUITableCell>
    //                                             <MUITableCell>{data.paymentDetails.swiftCode}</MUITableCell>
    //                                         </TableRow>
    //                                     </TableBody>
    //                                 </Table>
    //                             </TableContainer>
    //                         </div>
    //                     </Grid>
    //                 </Grid>
    //             </CardContent>
    //
    //             <Divider sx={{ mb: '0 !important' }} />
    //
    //             <TableContainer>
    //                 <Table>
    //                     <TableHead>
    //                         <TableRow>
    //                             <TableCell sx={{ py: 2 }}>Item</TableCell>
    //                             <TableCell sx={{ py: 2 }}>Description</TableCell>
    //                             <TableCell sx={{ py: 2 }}>hours</TableCell>
    //                             <TableCell sx={{ py: 2 }}>qty</TableCell>
    //                             <TableCell sx={{ py: 2 }}>Total</TableCell>
    //                         </TableRow>
    //                     </TableHead>
    //                     <TableBody>
    //                         <TableRow>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>
    //                                 Premium Branding Package
    //                             </TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>Branding & Promotion</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>48</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>1</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>$32</TableCell>
    //                         </TableRow>
    //                         <TableRow>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>Social Media</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>Social media templates</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>42</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>1</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>$28</TableCell>
    //                         </TableRow>
    //                         <TableRow>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>Web Design</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>Web designing package</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>46</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>1</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>$24</TableCell>
    //                         </TableRow>
    //                         <TableRow>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>SEO</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>
    //                                 Search engine optimization
    //                             </TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>40</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>1</TableCell>
    //                             <TableCell sx={{ py: theme => `${theme.spacing(2.75)} !important` }}>$22</TableCell>
    //                         </TableRow>
    //                     </TableBody>
    //                 </Table>
    //             </TableContainer>
    //
    //             <CardContent>
    //                 <Grid container sx={{ pt: 6, pb: 4 }}>
    //                     <Grid item xs={12} sm={7} lg={9} sx={{ order: { sm: 1, xs: 2 } }}>
    //                         <Box sx={{ mb: 2.5, display: 'flex', alignItems: 'center' }}>
    //                             <Typography sx={{ mr: 2, fontWeight: 600, color: 'text.secondary' }}>Salesperson:</Typography>
    //                             <Typography sx={{ color: 'text.secondary' }}>Tommy Shelby</Typography>
    //                         </Box>
    //
    //                         <Typography sx={{ color: 'text.secondary' }}>Thanks for your business</Typography>
    //                     </Grid>
    //                     <Grid item xs={12} sm={5} lg={3} sx={{ mb: { sm: 0, xs: 6 }, order: { sm: 2, xs: 1 } }}>
    //                         <CalcWrapper>
    //                             <Typography sx={{ color: 'text.secondary' }}>Subtotal:</Typography>
    //                             <Typography sx={{ fontWeight: 600, color: 'text.secondary' }}>$154.25</Typography>
    //                         </CalcWrapper>
    //                         <CalcWrapper>
    //                             <Typography sx={{ color: 'text.secondary' }}>Discount:</Typography>
    //                             <Typography sx={{ fontWeight: 600, color: 'text.secondary' }}>$00.00</Typography>
    //                         </CalcWrapper>
    //                         <CalcWrapper>
    //                             <Typography sx={{ color: 'text.secondary' }}>Tax:</Typography>
    //                             <Typography sx={{ fontWeight: 600, color: 'text.secondary' }}>$50.00</Typography>
    //                         </CalcWrapper>
    //                         <CalcWrapper>
    //                             <Typography sx={{ color: 'text.secondary' }}>Total:</Typography>
    //                             <Typography sx={{ fontWeight: 600, color: 'text.secondary' }}>$204.25</Typography>
    //                         </CalcWrapper>
    //                     </Grid>
    //                 </Grid>
    //             </CardContent>
    //
    //             <Divider
    //                 sx={{ mt: theme => `${theme.spacing(2)} !important`, mb: theme => `${theme.spacing(0.5)} !important` }}
    //             />
    //
    //             <CardContent>
    //                 <Typography sx={{ color: 'text.secondary' }}>
    //                     <strong>Note:</strong> It was a pleasure working with you and your team. We hope you will keep us in mind
    //                     for future freelance projects. Thank You!
    //                 </Typography>
    //             </CardContent>
    //         </Card>
    //     )
    // } else {
    //     return null
    // }
}

export default PreviewCard
