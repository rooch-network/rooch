// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import Radio from '@mui/material/Radio'
import Switch from '@mui/material/Switch'
import Divider from '@mui/material/Divider'
import { styled } from '@mui/material/styles'
import RadioGroup from '@mui/material/RadioGroup'
import Typography from '@mui/material/Typography'
import Box, { BoxProps } from '@mui/material/Box'
import FormControlLabel from '@mui/material/FormControlLabel'

// ** Type Import
import { Settings } from 'src/@core/context/settingsContext'

// ** Hook Import
import { useSettings } from 'src/@core/hooks/useSettings'

const CustomizerSpacing = styled('div')(({ theme }) => ({
  padding: theme.spacing(5, 6),
}))

const ColorBox = styled(Box)<BoxProps>(({ theme }) => ({
  width: 45,
  height: 45,
  cursor: 'pointer',
  margin: theme.spacing(2.5, 1.75, 1.75),
  borderRadius: theme.shape.borderRadius,
  transition:
    'margin .25s ease-in-out, width .25s ease-in-out, height .25s ease-in-out, box-shadow .25s ease-in-out',
  '&:hover': {
    boxShadow: theme.shadows[4],
  },
}))

const Customizer = () => {
  // ** Hook
  const { settings, saveSettings } = useSettings()

  // ** Vars
  const { mode, skin, direction, themeColor } = settings

  const handleChange = (field: keyof Settings, value: Settings[keyof Settings]): void => {
    saveSettings({ ...settings, [field]: value })
  }

  return (
    <div className="customizer">
      <Box
        className="customizer-header"
        sx={{
          position: 'relative',
          p: (theme) => theme.spacing(3.5, 5),
          borderBottom: (theme) => `1px solid ${theme.palette.divider}`,
        }}
      >
        <Typography variant="h6" sx={{ fontWeight: 600, textTransform: 'uppercase' }}>
          Theme
        </Typography>
      </Box>
      <CustomizerSpacing className="customizer-body">
        <Typography
          component="p"
          variant="caption"
          sx={{ mb: 5, color: 'text.disabled', textTransform: 'uppercase' }}
        >
          Theming
        </Typography>

        {/* Skin */}
        <Box sx={{ mb: 5 }}>
          <Typography>Skin</Typography>
          <RadioGroup
            row
            value={skin}
            onChange={(e) => handleChange('skin', e.target.value as Settings['skin'])}
            sx={{
              '& .MuiFormControlLabel-label': { fontSize: '.875rem', color: 'text.secondary' },
            }}
          >
            <FormControlLabel value="default" label="Default" control={<Radio />} />
            <FormControlLabel value="bordered" label="Bordered" control={<Radio />} />
          </RadioGroup>
        </Box>

        {/* Mode */}
        <Box sx={{ mb: 5 }}>
          <Typography>Mode</Typography>
          <RadioGroup
            row
            value={mode}
            onChange={(e) => handleChange('mode', e.target.value as any)}
            sx={{
              '& .MuiFormControlLabel-label': { fontSize: '.875rem', color: 'text.secondary' },
            }}
          >
            <FormControlLabel value="light" label="Light" control={<Radio />} />
            <FormControlLabel value="dark" label="Dark" control={<Radio />} />
          </RadioGroup>
        </Box>

        {/* Color Picker */}
        <div>
          <Typography>Primary Color</Typography>
          <Box sx={{ display: 'flex' }}>
            <ColorBox
              onClick={() => handleChange('themeColor', 'primary')}
              sx={{
                backgroundColor: '#696CFF',
                ...(themeColor === 'primary'
                  ? { width: 53, height: 53, m: (theme) => theme.spacing(1.5, 0.75, 0) }
                  : {}),
              }}
            />
            <ColorBox
              onClick={() => handleChange('themeColor', 'secondary')}
              sx={{
                backgroundColor: 'secondary.main',
                ...(themeColor === 'secondary'
                  ? { width: 53, height: 53, m: (theme) => theme.spacing(1.5, 0.75, 0) }
                  : {}),
              }}
            />
            <ColorBox
              onClick={() => handleChange('themeColor', 'success')}
              sx={{
                backgroundColor: 'success.main',
                ...(themeColor === 'success'
                  ? { width: 53, height: 53, m: (theme) => theme.spacing(1.5, 0.75, 0) }
                  : {}),
              }}
            />
            <ColorBox
              onClick={() => handleChange('themeColor', 'error')}
              sx={{
                backgroundColor: 'error.main',
                ...(themeColor === 'error'
                  ? { width: 53, height: 53, m: (theme) => theme.spacing(1.5, 0.75, 0) }
                  : {}),
              }}
            />
            <ColorBox
              onClick={() => handleChange('themeColor', 'warning')}
              sx={{
                backgroundColor: 'warning.main',
                ...(themeColor === 'warning'
                  ? { width: 53, height: 53, m: (theme) => theme.spacing(1.5, 0.75, 0) }
                  : {}),
              }}
            />
            <ColorBox
              onClick={() => handleChange('themeColor', 'info')}
              sx={{
                backgroundColor: 'info.main',
                ...(themeColor === 'info'
                  ? { width: 53, height: 53, m: (theme) => theme.spacing(1.5, 0.75, 0) }
                  : {}),
              }}
            />
          </Box>
        </div>
      </CustomizerSpacing>

      <Divider sx={{ m: '0 !important' }} />

      <CustomizerSpacing className="customizer-body">
        <Typography
          component="p"
          variant="caption"
          sx={{ mb: 5, color: 'text.disabled', textTransform: 'uppercase' }}
        >
          Misc
        </Typography>

        {/* RTL */}
        <Box sx={{ mb: 5, display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <Typography>RTL</Typography>
          <Switch
            name="direction"
            checked={direction === 'rtl'}
            onChange={(e) => handleChange('direction', e.target.checked ? 'rtl' : 'ltr')}
          />
        </Box>
      </CustomizerSpacing>
    </div>
  )
}

export default Customizer
