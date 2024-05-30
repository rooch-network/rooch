// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Next Imports
import Link from 'next/link'

// ** MUI Imports
import IconButton from '@mui/material/IconButton'
import Typography from '@mui/material/Typography'
import Box, { BoxProps } from '@mui/material/Box'
import { styled, useTheme } from '@mui/material/styles'

// ** Type Import
import { LayoutProps } from 'src/@core/layouts/types'

// ** Custom Icon Import
import Icon from 'src/@core/components/icon'

interface Props {
  navHover: boolean
  collapsedNavWidth: number
  hidden: LayoutProps['hidden']
  navigationBorderWidth: number
  toggleNavVisibility: () => void
  settings: LayoutProps['settings']
  saveSettings: LayoutProps['saveSettings']
  navMenuBranding?: LayoutProps['verticalLayoutProps']['navMenu']['branding']
  menuLockedIcon?: LayoutProps['verticalLayoutProps']['navMenu']['lockedIcon']
  menuUnlockedIcon?: LayoutProps['verticalLayoutProps']['navMenu']['unlockedIcon']
}

// ** Styled Components
const MenuHeaderWrapper = styled(Box)<BoxProps>(({ theme }) => ({
  display: 'flex',
  overflow: 'hidden',
  alignItems: 'center',
  marginTop: theme.spacing(3),
  paddingRight: theme.spacing(5),
  justifyContent: 'space-between',
  transition: 'padding .25s ease-in-out',
  minHeight: theme.mixins.toolbar.minHeight,
}))

const LinkStyled = styled(Link)({
  display: 'flex',
  alignItems: 'center',
  textDecoration: 'none',
})

const VerticalNavHeader = (props: Props) => {
  // ** Props
  const {
    hidden,
    navHover,
    settings,
    saveSettings,
    collapsedNavWidth,
    toggleNavVisibility,
    navigationBorderWidth,
    menuLockedIcon: userMenuLockedIcon,
    navMenuBranding: userNavMenuBranding,
    menuUnlockedIcon: userMenuUnlockedIcon,
  } = props

  // ** Hooks & Vars
  const theme = useTheme()
  const { skin, direction, navCollapsed } = settings

  const menuCollapsedStyles = navCollapsed && !navHover ? { opacity: 0 } : { opacity: 1 }

  const handleButtonClick = () => {
    if (hidden) {
      toggleNavVisibility()
    } else {
      saveSettings({ ...settings, navCollapsed: !navCollapsed })
    }
  }

  const menuHeaderPaddingLeft = () => {
    if (navCollapsed && !navHover) {
      if (userNavMenuBranding) {
        return 0
      } else {
        return (collapsedNavWidth - navigationBorderWidth - 22) / 8
      }
    } else {
      return 8
    }
  }

  const svgRotationDeg = () => {
    if (navCollapsed) {
      if (direction === 'rtl') {
        if (navHover) {
          return 0
        } else {
          return 180
        }
      } else {
        if (navHover) {
          return 180
        } else {
          return 0
        }
      }
    } else {
      if (direction === 'rtl') {
        return 180
      } else {
        return 0
      }
    }
  }

  return (
    <MenuHeaderWrapper className="nav-header" sx={{ pl: menuHeaderPaddingLeft() }}>
      {userNavMenuBranding ? (
        userNavMenuBranding(props)
      ) : (
        <LinkStyled href="/">
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

          <Typography
            variant="h5"
            sx={{
              lineHeight: 1,
              fontWeight: 700,
              ...menuCollapsedStyles,
              letterSpacing: '-0.45px',
              fontSize: '1.75rem !important',
              ...(navCollapsed && !navHover ? {} : { ml: 2 }),
              transition: 'opacity .35s ease-in-out, margin .35s ease-in-out',
            }}
          >
            <svg
              width="100"
              height="31.59"
              viewBox="0 0 343 78"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                fill={theme.palette.primary.main}
                d="M284.315 0.375V77.625H298.065V47.925C298.065 43.965 298.725 40.7017 300.045 38.135C301.438 35.495 303.381 33.515 305.875 32.195C308.368 30.875 311.228 30.215 314.455 30.215C318.855 30.215 322.265 31.4983 324.685 34.065C327.178 36.6317 328.425 40.5183 328.425 45.725V77.625H342.175V43.965C342.175 38.025 341.111 33.185 338.985 29.445C336.858 25.6317 333.925 22.8083 330.185 20.975C326.518 19.1417 322.338 18.225 317.645 18.225C312.511 18.225 307.965 19.2517 304.005 21.305C301.703 22.4774 299.723 23.9583 298.065 25.7479V0.375H284.315Z"
              />
              <path
                fill={theme.palette.primary.main}
                d="M0 77.625V0.625004H31.68C38.5 0.625004 44.33 1.72501 49.17 3.92501C54.0833 6.12501 57.86 9.27834 60.5 13.385C63.14 17.4917 64.46 22.3683 64.46 28.015C64.46 33.6617 63.14 38.5383 60.5 42.645C57.86 46.6783 54.0833 49.795 49.17 51.995C48.8266 52.1459 48.4782 52.2914 48.1248 52.4316L65.78 77.625H50.38L34.6923 55.1127C33.7106 55.1609 32.7065 55.185 31.68 55.185H14.3V77.625H0ZM14.3 43.905L31.02 43.415C37.3267 43.415 42.0567 42.0583 45.21 39.345C48.4367 36.6317 50.05 32.855 50.05 28.015C50.05 23.1017 48.4367 19.325 45.21 16.685C42.0567 14.045 37.3267 12.725 31.02 12.725H14.3V43.905Z"
              />
              <path
                fill={theme.palette.primary.main}
                d="M105.539 77.395C99.5257 77.395 94.1723 76.1117 89.479 73.545C84.7857 70.905 81.0823 67.3117 78.369 62.765C75.6557 58.2183 74.299 53.0483 74.299 47.255C74.299 41.3883 75.6557 36.2183 78.369 31.745C81.0823 27.1983 84.7857 23.6417 89.479 21.075C94.1723 18.5083 99.5257 17.225 105.539 17.225C111.626 17.225 117.016 18.5083 121.709 21.075C126.476 23.6417 130.179 27.1617 132.819 31.635C135.532 36.1083 136.889 41.315 136.889 47.255C136.889 53.0483 135.532 58.2183 132.819 62.765C130.179 67.3117 126.476 70.905 121.709 73.545C117.016 76.1117 111.626 77.395 105.539 77.395ZM105.539 65.625C108.912 65.625 111.919 64.8917 114.559 63.425C117.199 61.9583 119.252 59.8317 120.719 57.045C122.259 54.2583 123.029 50.995 123.029 47.255C123.029 43.4417 122.259 40.1783 120.719 37.465C119.252 34.6783 117.199 32.5517 114.559 31.085C111.919 29.6183 108.949 28.885 105.649 28.885C102.276 28.885 99.269 29.6183 96.629 31.085C94.0623 32.5517 92.009 34.6783 90.469 37.465C88.929 40.1783 88.159 43.4417 88.159 47.255C88.159 50.995 88.929 54.2583 90.469 57.045C92.009 59.8317 94.0623 61.9583 96.629 63.425C99.269 64.8917 102.239 65.625 105.539 65.625Z"
              />
              <path
                fill={theme.palette.primary.main}
                d="M160.377 73.545C165.071 76.1117 170.424 77.395 176.437 77.395C182.524 77.395 187.914 76.1117 192.607 73.545C197.374 70.905 201.077 67.3117 203.717 62.765C206.431 58.2183 207.787 53.0483 207.787 47.255C207.787 41.315 206.431 36.1083 203.717 31.635C201.077 27.1617 197.374 23.6417 192.607 21.075C187.914 18.5083 182.524 17.225 176.437 17.225C170.424 17.225 165.071 18.5083 160.377 21.075C155.684 23.6417 151.981 27.1983 149.267 31.745C146.554 36.2183 145.197 41.3883 145.197 47.255C145.197 53.0483 146.554 58.2183 149.267 62.765C151.981 67.3117 155.684 70.905 160.377 73.545ZM185.457 63.425C182.817 64.8917 179.811 65.625 176.437 65.625C173.137 65.625 170.167 64.8917 167.527 63.425C164.961 61.9583 162.907 59.8317 161.367 57.045C159.827 54.2583 159.057 50.995 159.057 47.255C159.057 43.4417 159.827 40.1783 161.367 37.465C162.907 34.6783 164.961 32.5517 167.527 31.085C170.167 29.6183 173.174 28.885 176.547 28.885C179.847 28.885 182.817 29.6183 185.457 31.085C188.097 32.5517 190.151 34.6783 191.617 37.465C193.157 40.1783 193.927 43.4417 193.927 47.255C193.927 50.995 193.157 54.2583 191.617 57.045C190.151 59.8317 188.097 61.9583 185.457 63.425Z"
              />
              <path
                fill={theme.palette.primary.main}
                d="M247.886 77.395C241.726 77.395 236.226 76.1117 231.386 73.545C226.619 70.905 222.879 67.3117 220.166 62.765C217.453 58.2183 216.096 53.0483 216.096 47.255C216.096 41.3883 217.453 36.2183 220.166 31.745C222.879 27.1983 226.619 23.6417 231.386 21.075C236.226 18.5083 241.726 17.225 247.886 17.225C253.606 17.225 258.629 18.3983 262.956 20.745C267.356 23.0183 270.693 26.3917 272.966 30.865L262.406 37.025C260.646 34.2383 258.483 32.185 255.916 30.865C253.423 29.545 250.709 28.885 247.776 28.885C244.403 28.885 241.359 29.6183 238.646 31.085C235.933 32.5517 233.806 34.6783 232.266 37.465C230.726 40.1783 229.956 43.4417 229.956 47.255C229.956 51.0683 230.726 54.3683 232.266 57.155C233.806 59.8683 235.933 61.9583 238.646 63.425C241.359 64.8917 244.403 65.625 247.776 65.625C250.709 65.625 253.423 64.965 255.916 63.645C258.483 62.325 260.646 60.2717 262.406 57.485L272.966 63.645C270.693 68.045 267.356 71.455 262.956 73.875C258.629 76.2217 253.606 77.395 247.886 77.395Z"
              />
            </svg>
          </Typography>
        </LinkStyled>
      )}

      {userMenuLockedIcon === null && userMenuUnlockedIcon === null ? null : (
        <IconButton
          disableRipple
          disableFocusRipple
          onClick={handleButtonClick}
          sx={{
            p: 1.75,
            right: -19,
            position: 'absolute',
            color: 'text.primary',
            '& svg': { color: 'common.white' },
            transition: 'right .25s ease-in-out',
            backgroundColor: hidden ? 'background.paper' : 'customColors.collapseTogglerBg',
            ...(navCollapsed && !navHover && { display: 'none' }),
            ...(!hidden &&
              skin === 'bordered' && {
                '&:before': {
                  zIndex: -1,
                  content: '""',
                  width: '105%',
                  height: '105%',
                  borderRadius: '50%',
                  position: 'absolute',
                  border: `1px solid ${theme.palette.divider}`,
                  clipPath:
                    direction === 'rtl' ? 'circle(71% at 100% 50%)' : 'circle(71% at 0% 50%)',
                },
              }),
          }}
        >
          <Box sx={{ display: 'flex', borderRadius: 5, backgroundColor: 'primary.main' }}>
            {userMenuLockedIcon && userMenuUnlockedIcon ? (
              navCollapsed ? (
                userMenuUnlockedIcon
              ) : (
                userMenuLockedIcon
              )
            ) : (
              <Box
                sx={{
                  display: 'flex',
                  '& svg': {
                    transform: `rotate(${svgRotationDeg()}deg)`,
                    transition: 'transform .25s ease-in-out .35s',
                  },
                }}
              >
                <Icon icon="bx:chevron-left" />
              </Box>
            )}
          </Box>
        </IconButton>
      )}
    </MenuHeaderWrapper>
  )
}

export default VerticalNavHeader
