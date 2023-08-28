// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { ReactNode, useState } from 'react'

// ** Next Import
import Link from 'next/link'

// ** MUI Components
import Box from '@mui/material/Box'
import Card from '@mui/material/Card'
import Button from '@mui/material/Button'
import Divider from '@mui/material/Divider'
import TextField from '@mui/material/TextField'
import InputLabel from '@mui/material/InputLabel'
import Typography from '@mui/material/Typography'
import IconButton from '@mui/material/IconButton'
import CardContent from '@mui/material/CardContent'
import FormControl from '@mui/material/FormControl'
import { styled, useTheme } from '@mui/material/styles'
import MenuItem from '@mui/material/MenuItem'
import Select, { SelectChangeEvent } from '@mui/material/Select'
import FormControlLabel from '@mui/material/FormControlLabel'
import FormHelperText from '@mui/material/FormHelperText'

// ** Third Party Imports
import * as yup from 'yup'
import { useForm, Controller } from 'react-hook-form'
import { yupResolver } from '@hookform/resolvers/yup'

// ** Icon Imports
import Icon from 'src/@core/components/icon'

// ** Configs
import themeConfig from 'src/configs/themeConfig'

// ** Layout Import
import BlankLayout from 'src/@core/layouts/BlankLayout'

// ** Hooks
import { useAuth } from 'src/hooks/useAuth'

//import useBgColor from 'src/@core/hooks/useBgColor'
//import { useSettings } from 'src/@core/hooks/useSettings'

// ** Demo Imports
import AuthIllustrationWrapper from 'src/views/pages/auth/AuthIllustrationWrapper'

// ** Styled Components
const LinkStyled = styled(Link)(({ theme }) => ({
  fontSize: '0.875rem',
  textDecoration: 'none',
  color: theme.palette.primary.main,
}))

const schema = yup.object().shape({
  secretKey: yup.string().min(43).required(),
})

const defaultValues = {
  secretKey: 'AM4KesRCz7SzQt+F9TK0IvznFGxjUWGgRNlJxbTLW0Ol',
}

interface FormData {
  secretKey: string
}

enum InputType {
  Connect,
  Import,
  Create,
}

const LoginPage = () => {
  // hooks
  const auth = useAuth()
  const theme = useTheme()

  //  const { settings } = useSettings()
  //  const bgColors = useBgColor()

  const {
    control,

    //    setError,
    handleSubmit,
    formState: { errors },
  } = useForm({
    defaultValues,
    mode: 'onBlur',
    resolver: yupResolver(schema),
  })

  const [inputType, setInputType] = useState<InputType>(InputType.Connect)

  // ** State
  const [statusValue, setStatusValue] = useState<string>('')

  const handleStatusValue = (e: SelectChangeEvent) => {
    setStatusValue(e.target.value)
  }

  const onSubmit = (data: FormData) => {
    const { secretKey } = data

    console.log(secretKey)

    auth.loginByMetamask()

    //    auth.login({ email, password, rememberMe: true }, () => {
    //      setError('email', {
    //        type: 'manual',
    //        message: 'Email or Password is invalid'
    //      })
    //    })
  }

  return (
    <Box className="content-center">
      <AuthIllustrationWrapper>
        <Card>
          <CardContent sx={{ p: `${theme.spacing(8, 8, 7)} !important` }}>
            <Box sx={{ mb: 8, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
              <Typography
                variant="h5"
                sx={{
                  ml: 2,
                  lineHeight: 1,
                  fontWeight: 700,
                  letterSpacing: '-0.45px',
                  fontSize: '1.75rem !important',
                }}
              >
                {themeConfig.templateName} Dashboard
              </Typography>
            </Box>
            <Typography variant="h6" sx={{ mb: 1.5 }}>
              Welcome to {themeConfig.templateName} Dashboard!!!! 👋🏻
            </Typography>
            <Typography sx={{ mb: 6, color: 'text.secondary' }}>
              Please connect to your account and start the adventure
            </Typography>
            <form noValidate autoComplete="off" onSubmit={handleSubmit(onSubmit)}>
              {inputType === InputType.Import ? (
                <FormControl fullWidth sx={{ mb: 2 }}>
                  <Controller
                    name="secretKey"
                    control={control}
                    rules={{ required: true }}
                    render={({ field: { value, onChange, onBlur } }) => (
                      <TextField
                        autoFocus
                        label="Secret Key"
                        value={value}
                        onBlur={onBlur}
                        onChange={onChange}
                        error={Boolean(errors.secretKey)}
                        placeholder=""
                      />
                    )}
                  />
                  {errors.secretKey && (
                    <FormHelperText sx={{ color: 'error.main' }}>
                      {errors.secretKey.message}
                    </FormHelperText>
                  )}
                </FormControl>
              ) : (
                <>
                  <FormControl fullWidth>
                    <InputLabel id="invoice-status-select">Select Wallet</InputLabel>
                    <Select
                      fullWidth
                      value={statusValue}
                      sx={{ mr: 4, mb: 2 }}
                      label="Select Wallet"
                      onChange={handleStatusValue}
                      labelId="invoice-status-select"
                    >
                      <MenuItem value="Bitcoin">Bitcoin</MenuItem>
                      <MenuItem value="Matemask">Matemask</MenuItem>
                    </Select>
                  </FormControl>
                </>
              )}
              <Box
                sx={{
                  mb: 4,
                  display: 'flex',
                  alignItems: 'center',
                  flexWrap: 'wrap',
                  justifyContent: 'space-between',
                }}
              >
                <FormControlLabel
                  label=""
                  control={<></>}
                  sx={{
                    '& .MuiFormControlLabel-label': {
                      fontSize: '0.875rem',
                      color: 'text.secondary',
                    },
                  }}
                />
                <Button
                  onClick={() => {
                    if (inputType === InputType.Connect) {
                      setInputType(InputType.Import)
                    } else {
                      setInputType(InputType.Connect)
                    }
                    console.log('hhahah')
                  }}
                >
                  {inputType === InputType.Import ? 'Select Account' : 'Import Account'}
                </Button>
              </Box>
              <Button fullWidth size="large" type="submit" variant="contained" sx={{ mb: 4 }}>
                Connect
              </Button>
              <Box
                sx={{
                  display: 'flex',
                  alignItems: 'center',
                  flexWrap: 'wrap',
                  justifyContent: 'center',
                }}
              >
                <Typography variant="body2" sx={{ mr: 2 }}>
                  New on our platform?
                </Typography>
                <Typography>
                  <LinkStyled href="/">Create an account</LinkStyled>
                </Typography>
              </Box>
              <Divider sx={{ my: `${theme.spacing(6)} !important` }}>or</Divider>
              <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                <IconButton
                  href="/"
                  component={Link}
                  sx={{ color: '#497ce2' }}
                  onClick={(e) => e.preventDefault()}
                >
                  <Icon icon="bxl:facebook-circle" />
                </IconButton>
                <IconButton
                  href="/"
                  component={Link}
                  sx={{ color: '#1da1f2' }}
                  onClick={(e) => e.preventDefault()}
                >
                  <Icon icon="bxl:twitter" />
                </IconButton>
                <IconButton
                  href="/"
                  component={Link}
                  onClick={(e) => e.preventDefault()}
                  sx={{ color: theme.palette.mode === 'light' ? '#272727' : 'grey.300' }}
                >
                  <Icon icon="bxl:github" />
                </IconButton>
                <IconButton
                  href="/"
                  component={Link}
                  sx={{ color: '#db4437' }}
                  onClick={(e) => e.preventDefault()}
                >
                  <Icon icon="bxl:google" />
                </IconButton>
              </Box>
            </form>
          </CardContent>
        </Card>
      </AuthIllustrationWrapper>
    </Box>
  )
}

LoginPage.getLayout = (page: ReactNode) => <BlankLayout>{page}</BlankLayout>

LoginPage.guestGuard = true

export default LoginPage
