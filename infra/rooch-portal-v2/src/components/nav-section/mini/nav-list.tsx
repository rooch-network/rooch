import { useRef, useState, useEffect, useCallback } from 'react';

import Paper from '@mui/material/Paper';
import Popover from '@mui/material/Popover';
import { useTheme } from '@mui/material/styles';

import { usePathname } from 'src/routes/hooks';
import { isExternalLink } from 'src/routes/utils';
import { useActiveLink } from 'src/routes/hooks/use-active-link';

import { paper } from 'src/theme/styles';

import { NavItem } from './nav-item';
import { NavUl, NavLi } from '../styles';
import { navSectionClasses } from '../classes';

import type { NavListProps, NavSubListProps } from '../types';

export function NavList({
  data,
  depth,
  render,
  cssVars,
  slotProps,
  enabledRootRedirect,
}: NavListProps) {
  const theme = useTheme();

  const pathname = usePathname();

  const navItemRef = useRef<HTMLButtonElement | null>(null);

  const active = useActiveLink(data.path, !!data.children);

  const [openMenu, setOpenMenu] = useState(false);

  useEffect(() => {
    if (openMenu) {
      handleCloseMenu();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pathname]);

  const handleOpenMenu = useCallback(() => {
    if (data.children) {
      setOpenMenu(true);
    }
  }, [data.children]);

  const handleCloseMenu = useCallback(() => {
    setOpenMenu(false);
  }, []);

  const renderNavItem = (
    <NavItem
      ref={navItemRef}
      render={render}
      // slots
      path={data.path}
      icon={data.icon}
      info={data.info}
      title={data.title}
      caption={data.caption}
      // state
      depth={depth}
      active={active}
      disabled={data.disabled}
      hasChild={!!data.children}
      open={data.children && openMenu}
      externalLink={isExternalLink(data.path)}
      enabledRootRedirect={enabledRootRedirect}
      // styles
      slotProps={depth === 1 ? slotProps?.rootItem : slotProps?.subItem}
      // actions
      onMouseEnter={handleOpenMenu}
      onMouseLeave={handleCloseMenu}
    />
  );

  // Hidden item by role
  if (data.roles && slotProps?.currentRole) {
    if (!data?.roles?.includes(slotProps?.currentRole)) {
      return null;
    }
  }

  // Has children
  if (data.children) {
    return (
      <NavLi disabled={data.disabled}>
        {renderNavItem}

        <Popover
          disableScrollLock
          open={openMenu}
          anchorEl={navItemRef.current}
          anchorOrigin={{ vertical: 'center', horizontal: 'right' }}
          transformOrigin={{ vertical: 'center', horizontal: 'left' }}
          slotProps={{
            paper: {
              onMouseEnter: handleOpenMenu,
              onMouseLeave: handleCloseMenu,
              sx: {
                px: 0.75,
                boxShadow: 'none',
                overflow: 'unset',
                backdropFilter: 'none',
                background: 'transparent',
                ...(depth > 1 && { mt: -1 }),
                ...(openMenu && { pointerEvents: 'auto' }),
              },
            },
          }}
          sx={{ ...cssVars, pointerEvents: 'none' }}
        >
          <Paper
            className={navSectionClasses.paper}
            sx={{ minWidth: 180, ...paper({ theme, dropdown: true }), ...slotProps?.paper }}
          >
            <NavSubList
              data={data.children}
              depth={depth}
              render={render}
              cssVars={cssVars}
              slotProps={slotProps}
              enabledRootRedirect={enabledRootRedirect}
            />
          </Paper>
        </Popover>
      </NavLi>
    );
  }

  // Default
  return <NavLi disabled={data.disabled}>{renderNavItem}</NavLi>;
}

function NavSubList({
  data,
  render,
  depth,
  slotProps,
  enabledRootRedirect,
  cssVars,
}: NavSubListProps) {
  return (
    <NavUl sx={{ gap: 0.5 }}>
      {data.map((list) => (
        <NavList
          key={list.title}
          data={list}
          render={render}
          depth={depth + 1}
          cssVars={cssVars}
          slotProps={slotProps}
          enabledRootRedirect={enabledRootRedirect}
        />
      ))}
    </NavUl>
  );
}
