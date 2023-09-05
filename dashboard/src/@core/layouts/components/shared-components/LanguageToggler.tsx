// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Mui Imports
import Button from '@mui/material/Button'

// ** Icon Imports
import Icon from 'src/@core/components/icon'

// ** Third Party Import
import { useTranslation } from 'react-i18next'

// ** Types Import
import { Settings } from 'src/@core/context/settingsContext'

interface Props {
  settings: Settings
  saveSettings: (values: Settings) => void
}

const LanguageToggler = (props: Props) => {
  // ** Props
  const { settings, saveSettings } = props

  // ** Hook
  const { i18n } = useTranslation()

  const handleLangItemClick = () => {
    if (i18n.language == 'en') {
      i18n.changeLanguage('en')
    } else {
      i18n.changeLanguage('cn')
    }

  }

  return (
    <Button color="inherit" aria-haspopup="true" onClick={handleLangItemClick}>
      {i18n.language === 'en' ? 'EN':'CN'}
    </Button>
  )
}

export default LanguageToggler
