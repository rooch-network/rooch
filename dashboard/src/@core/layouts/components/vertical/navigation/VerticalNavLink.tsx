// ** React Imports
import { ElementType } from 'react'

// ** Next Imports
import Link from 'next/link'
import { useRouter } from 'next/router'

// ** MUI Imports
import Chip from '@mui/material/Chip'
import ListItem from '@mui/material/ListItem'
import { styled } from '@mui/material/styles'
import Typography from '@mui/material/Typography'
import Box, { BoxProps } from '@mui/material/Box'
import ListItemIcon from '@mui/material/ListItemIcon'
import ListItemButton, { ListItemButtonProps } from '@mui/material/ListItemButton'

// ** Configs Import
import themeConfig from 'src/configs/themeConfig'

// ** Types
import { NavLink, NavGroup } from 'src/@core/layouts/types'
import { Settings } from 'src/@core/context/settingsContext'

// ** Custom Components Imports
import UserIcon from 'src/layouts/components/UserIcon'
import Translations from 'src/layouts/components/Translations'
import CanViewNavLink from 'src/layouts/components/acl/CanViewNavLink'

// ** Hook Import
import useBgColor, { UseBgColorType } from 'src/@core/hooks/useBgColor'

// ** Util Import
import { hexToRGBA } from 'src/@core/utils/hex-to-rgba'
import { handleURLQueries } from 'src/@core/layouts/utils'

interface Props {
  parent?: boolean
  item: NavLink
  navHover?: boolean
  settings: Settings
  navVisible?: boolean
  collapsedNavWidth: number
  navigationBorderWidth: number
  toggleNavVisibility: () => void
  isSubToSub?: NavGroup | undefined
}

// ** Styled Components
const MenuNavLink = styled(ListItemButton)<
  ListItemButtonProps & { component?: ElementType; href: string; target?: '_blank' | undefined }
>(({ theme }) => ({
  width: '100%',
  margin: theme.spacing(0, 4),
  transition: 'padding .25s ease-in-out',
  borderRadius: theme.shape.borderRadius
}))

const MenuItemTextMetaWrapper = styled(Box)<BoxProps>(({ theme }) => ({
  width: '100%',
  display: 'flex',
  alignItems: 'center',
  gap: theme.spacing(2),
  justifyContent: 'space-between',
  transition: 'opacity .25s ease-in-out',
  ...(themeConfig.menuTextTruncate && { overflow: 'hidden' })
}))

const VerticalNavLink = ({
  item,
  parent,
  navHover,
  settings,
  navVisible,
  isSubToSub,
  collapsedNavWidth,
  toggleNavVisibility,
  navigationBorderWidth
}: Props) => {
  // ** Hooks
  const router = useRouter()
  const bgColors: UseBgColorType = useBgColor()

  // ** Vars
  const { mode, navCollapsed } = settings

  const icon = parent && !item.icon ? themeConfig.navSubItemIcon : item.icon

  const isNavLinkActive = () => {
    if (router.pathname === item.path || handleURLQueries(router, item.path)) {
      return true
    } else {
      return false
    }
  }

  return (
    <CanViewNavLink navLink={item}>
      <ListItem
        disablePadding
        className='nav-link'
        disabled={item.disabled || false}
        sx={{
          px: '0 !important',
          ...(!parent && {
            mt: 0.5,
            ...(isNavLinkActive() && {
              '&:before': {
                right: 0,
                width: 4,
                height: 42,
                content: '""',
                position: 'absolute',
                backgroundColor: 'primary.main',
                borderTopLeftRadius: theme => theme.shape.borderRadius,
                borderBottomLeftRadius: theme => theme.shape.borderRadius
              }
            })
          })
        }}
      >
        <MenuNavLink
          component={Link}
          {...(item.disabled && { tabIndex: -1 })}
          className={isNavLinkActive() ? 'active' : ''}
          href={item.path === undefined ? '/' : `${item.path}`}
          {...(item.openInNewTab ? { target: '_blank' } : null)}
          onClick={e => {
            if (item.path === undefined) {
              e.preventDefault()
              e.stopPropagation()
            }
            if (navVisible) {
              toggleNavVisibility()
            }
          }}
          sx={{
            py: 2.5,
            ...(item.disabled ? { pointerEvents: 'none' } : { cursor: 'pointer' }),
            pr: navCollapsed && !navHover ? ((collapsedNavWidth - navigationBorderWidth - 22) / 4 - 8) / 2 : 4,
            pl:
              navCollapsed && !navHover
                ? ((collapsedNavWidth - navigationBorderWidth - 22) / 4 - 8) / 2
                : parent
                ? 6
                : 4,
            ...(parent
              ? {
                  '&.active': {
                    '& .MuiTypography-root': {
                      fontWeight: 600,
                      color: 'text.primary'
                    },
                    '& svg': {
                      color: 'primary.main',
                      transform: 'scale(1.35)',
                      filter: theme => `drop-shadow(0 0 2px ${theme.palette.primary.main})`
                    }
                  }
                }
              : {
                  '&.active': {
                    backgroundColor: mode === 'light' ? bgColors.primaryLight.backgroundColor : 'primary.main',
                    '& .MuiTypography-root, & svg': {
                      color: mode === 'light' ? 'primary.main' : 'common.white'
                    },
                    '&.active.Mui-focusVisible': {
                      '&, &:hover': {
                        backgroundColor: theme =>
                          mode === 'light' ? hexToRGBA(theme.palette.primary.main, 0.24) : 'primary.dark'
                      }
                    }
                  }
                })
          }}
        >
          <ListItemIcon
            sx={{
              transition: 'margin .25s ease-in-out',
              '& svg': { transition: 'transform .25s ease-in-out' },
              ...(navCollapsed && !navHover ? { mr: 0 } : { mr: 2.5 }),
              ...(parent && { mr: 4.25, color: 'text.disabled' })
            }}
          >
            <UserIcon icon={icon as string} fontSize={parent ? '0.4375rem' : '1.375rem'} />
          </ListItemIcon>

          <MenuItemTextMetaWrapper
            sx={{
              ...(isSubToSub ? { ml: 2.5 } : {}),
              ...(navCollapsed && !navHover ? { opacity: 0 } : { opacity: 1 })
            }}
          >
            <Typography
              sx={{ color: 'text.secondary' }}
              {...((themeConfig.menuTextTruncate || (!themeConfig.menuTextTruncate && navCollapsed && !navHover)) && {
                noWrap: true
              })}
            >
              <Translations text={item.title} />
            </Typography>
            {item.badgeContent ? (
              <Chip
                label={item.badgeContent}
                color={item.badgeColor || 'primary'}
                sx={{
                  height: 20,
                  fontWeight: 500,
                  '& .MuiChip-label': { px: 1.5, textTransform: 'capitalize' }
                }}
              />
            ) : null}
          </MenuItemTextMetaWrapper>
        </MenuNavLink>
      </ListItem>
    </CanViewNavLink>
  )
}

export default VerticalNavLink
