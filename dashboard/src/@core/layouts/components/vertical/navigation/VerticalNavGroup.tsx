// ** React Imports
import { useEffect, Fragment } from 'react'

// ** Next Import
import { useRouter } from 'next/router'

// ** MUI Imports
import Chip from '@mui/material/Chip'
import Collapse from '@mui/material/Collapse'
import ListItem from '@mui/material/ListItem'
import { styled } from '@mui/material/styles'
import Typography from '@mui/material/Typography'
import Box, { BoxProps } from '@mui/material/Box'
import ListItemIcon from '@mui/material/ListItemIcon'
import ListItemButton from '@mui/material/ListItemButton'

// ** Third Party Imports
import clsx from 'clsx'

// ** Icon Imports
import Icon from 'src/@core/components/icon'

// ** Configs Import
import themeConfig from 'src/configs/themeConfig'

// ** Type Import
import { NavGroup, LayoutProps } from 'src/@core/layouts/types'

// ** Custom Components Imports
import VerticalNavItems from './VerticalNavItems'
import UserIcon from 'src/layouts/components/UserIcon'
import Translations from 'src/layouts/components/Translations'
import CanViewNavGroup from 'src/layouts/components/acl/CanViewNavGroup'

// ** Hook Import
import useBgColor, { UseBgColorType } from 'src/@core/hooks/useBgColor'

// ** Util Imports
import { hexToRGBA } from 'src/@core/utils/hex-to-rgba'
import { hasActiveChild, removeChildren } from 'src/@core/layouts/utils'

interface Props {
  item: NavGroup
  navHover: boolean
  parent?: NavGroup
  navVisible?: boolean
  groupActive: string[]
  collapsedNavWidth: number
  currentActiveGroup: string[]
  navigationBorderWidth: number
  settings: LayoutProps['settings']
  isSubToSub?: NavGroup | undefined
  saveSettings: LayoutProps['saveSettings']
  setGroupActive: (values: string[]) => void
  setCurrentActiveGroup: (items: string[]) => void
}

const MenuItemTextWrapper = styled(Box)<BoxProps>(({ theme }) => ({
  width: '100%',
  display: 'flex',
  alignItems: 'center',
  gap: theme.spacing(2),
  justifyContent: 'space-between',
  transition: 'opacity .25s ease-in-out',
  ...(themeConfig.menuTextTruncate && { overflow: 'hidden' })
}))

const VerticalNavGroup = (props: Props) => {
  // ** Props
  const {
    item,
    parent,
    settings,
    navHover,
    navVisible,
    isSubToSub,
    groupActive,
    setGroupActive,
    collapsedNavWidth,
    currentActiveGroup,
    setCurrentActiveGroup,
    navigationBorderWidth
  } = props

  // ** Hooks & Vars
  const router = useRouter()
  const currentURL = router.asPath
  const bgColors: UseBgColorType = useBgColor()
  const { direction, mode, navCollapsed, verticalNavToggleType } = settings

  // ** Accordion menu group open toggle
  const toggleActiveGroup = (item: NavGroup, parent: NavGroup | undefined) => {
    let openGroup = groupActive

    // ** If Group is already open and clicked, close the group
    if (openGroup.includes(item.title)) {
      openGroup.splice(openGroup.indexOf(item.title), 1)

      // If clicked Group has open group children, Also remove those children to close those groups
      if (item.children) {
        removeChildren(item.children, openGroup, currentActiveGroup)
      }
    } else if (parent) {
      // ** If Group clicked is the child of an open group, first remove all the open groups under that parent
      if (parent.children) {
        removeChildren(parent.children, openGroup, currentActiveGroup)
      }

      // ** After removing all the open groups under that parent, add the clicked group to open group array
      if (!openGroup.includes(item.title)) {
        openGroup.push(item.title)
      }
    } else {
      // ** If clicked on another group that is not active or open, create openGroup array from scratch

      // ** Empty Open Group array
      openGroup = []

      // ** push Current Active Group To Open Group array
      if (currentActiveGroup.every(elem => groupActive.includes(elem))) {
        openGroup.push(...currentActiveGroup)
      }

      // ** Push current clicked group item to Open Group array
      if (!openGroup.includes(item.title)) {
        openGroup.push(item.title)
      }
    }
    setGroupActive([...openGroup])
  }

  // ** Menu Group Click
  const handleGroupClick = () => {
    const openGroup = groupActive
    if (verticalNavToggleType === 'collapse') {
      if (openGroup.includes(item.title)) {
        openGroup.splice(openGroup.indexOf(item.title), 1)
      } else {
        openGroup.push(item.title)
      }
      setGroupActive([...openGroup])
    } else {
      toggleActiveGroup(item, parent)
    }
  }

  useEffect(() => {
    if (hasActiveChild(item, currentURL)) {
      if (!groupActive.includes(item.title)) groupActive.push(item.title)
    } else {
      const index = groupActive.indexOf(item.title)
      if (index > -1) groupActive.splice(index, 1)
    }
    setGroupActive([...groupActive])
    setCurrentActiveGroup([...groupActive])

    // Empty Active Group When Menu is collapsed and not hovered, to fix issue route change
    if (navCollapsed && !navHover) {
      setGroupActive([])
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [router.asPath])

  useEffect(() => {
    if (navCollapsed && !navHover) {
      setGroupActive([])
    }

    if ((navCollapsed && navHover) || (groupActive.length === 0 && !navCollapsed)) {
      setGroupActive([...currentActiveGroup])
    }

    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [navCollapsed, navHover])

  useEffect(() => {
    if (groupActive.length === 0 && !navCollapsed) {
      setGroupActive([])
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [navHover])

  const icon = parent && !item.icon ? themeConfig.navSubItemIcon : item.icon

  const menuGroupCollapsedStyles = navCollapsed && !navHover ? { opacity: 0 } : { opacity: 1 }

  return (
    <CanViewNavGroup navGroup={item}>
      <Fragment>
        <ListItem
          disablePadding
          className='nav-group'
          sx={{
            px: '0 !important',
            flexDirection: 'column',
            ...(!parent && {
              mt: 0.5,
              ...(currentActiveGroup.includes(item.title) && {
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
          <ListItemButton
            onClick={handleGroupClick}
            className={clsx({
              'Mui-selected': currentActiveGroup.includes(item.title)
            })}
            sx={{
              mx: 4,
              py: 2.5,
              borderRadius: 1,
              transition: 'padding .25s ease-in-out',
              width: theme => `calc(100% - ${theme.spacing(4 * 2)})`,
              pr: navCollapsed && !navHover ? ((collapsedNavWidth - navigationBorderWidth - 22) / 4 - 8) / 2 : 2.5,
              pl:
                navCollapsed && !navHover
                  ? ((collapsedNavWidth - navigationBorderWidth - 22) / 4 - 8) / 2
                  : parent
                  ? 6
                  : 4,
              ...(groupActive.includes(item.title) && {
                backgroundColor: 'action.hover'
              }),
              ...(groupActive.includes(item.title) && {
                '& .MuiTypography-root, & svg': {
                  color: 'text.primary'
                }
              }),
              '&.Mui-selected.Mui-focusVisible': {
                '&, &:hover': {
                  backgroundColor: theme =>
                    mode === 'light' ? hexToRGBA(theme.palette.primary.main, 0.24) : 'primary.dark'
                }
              },
              '&.Mui-selected': {
                '&, &:hover': {
                  backgroundColor: mode === 'light' ? bgColors.primaryLight.backgroundColor : 'primary.main'
                },
                '& .MuiTypography-root, & svg': {
                  color: mode === 'light' ? 'primary.main' : 'common.white'
                },
                '& + .MuiCollapse-root .Mui-selected': {
                  '&, &:hover': {
                    backgroundColor: 'action.hover'
                  },
                  '& .MuiTypography-root, & svg': { color: 'text.primary' }
                }
              }
            }}
          >
            {isSubToSub ? null : (
              <ListItemIcon
                sx={{
                  transition: 'margin .25s ease-in-out',
                  ...(parent && navCollapsed && !navHover ? {} : { mr: 2.5 }),
                  ...(navCollapsed && !navHover ? { mr: 0 } : {}), // this condition should come after (parent && navCollapsed && !navHover) condition for proper styling
                  ...(parent && { mr: 4.25, color: 'text.disabled' })
                }}
              >
                <UserIcon icon={icon as string} fontSize={parent ? '0.4375rem' : '1.375rem'} />
              </ListItemIcon>
            )}
            <MenuItemTextWrapper sx={{ ...menuGroupCollapsedStyles, ...(isSubToSub ? { ml: 9 } : {}) }}>
              <Typography
                sx={{ color: 'text.secondary' }}
                {...((themeConfig.menuTextTruncate || (!themeConfig.menuTextTruncate && navCollapsed && !navHover)) && {
                  noWrap: true
                })}
              >
                <Translations text={item.title} />
              </Typography>
              <Box
                className='menu-item-meta'
                sx={{
                  display: 'flex',
                  alignItems: 'center',
                  '& svg': {
                    color: 'text.secondary',
                    transition: 'transform .25s ease-in-out',
                    ...(groupActive.includes(item.title) && {
                      transform: direction === 'ltr' ? 'rotate(90deg)' : 'rotate(-90deg)'
                    })
                  }
                }}
              >
                {item.badgeContent ? (
                  <Chip
                    label={item.badgeContent}
                    color={item.badgeColor || 'primary'}
                    sx={{
                      mr: 1.5,
                      height: 20,
                      fontWeight: 500,
                      '& .MuiChip-label': { px: 1.5, textTransform: 'capitalize' }
                    }}
                  />
                ) : null}
                <Icon fontSize='1.25rem' icon={direction === 'ltr' ? 'bx:chevron-right' : 'bx:chevron-left'} />
              </Box>
            </MenuItemTextWrapper>
          </ListItemButton>
          <Collapse
            component='ul'
            onClick={e => e.stopPropagation()}
            in={groupActive.includes(item.title)}
            sx={{
              pl: 0,
              width: '100%',
              ...menuGroupCollapsedStyles,
              '& .MuiCollapse-wrapper': { pt: 1 }
            }}
          >
            <VerticalNavItems
              {...props}
              parent={item}
              navVisible={navVisible}
              verticalNavItems={item.children}
              isSubToSub={parent && item.children ? item : undefined}
            />
          </Collapse>
        </ListItem>
      </Fragment>
    </CanViewNavGroup>
  )
}

export default VerticalNavGroup
