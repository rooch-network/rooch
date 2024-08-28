import Stack from '@mui/material/Stack';
import { useTheme } from '@mui/material/styles';

import { NavList } from './nav-list';
import { NavUl, NavLi } from '../styles';
import { Scrollbar } from '../../scrollbar';
import { navSectionClasses } from '../classes';
import { navSectionCssVars } from '../css-vars';

import type { NavGroupProps, NavSectionProps } from '../types';

export function NavSectionHorizontal({
  sx,
  data,
  render,
  slotProps,
  enabledRootRedirect,
  cssVars: overridesVars,
}: NavSectionProps) {
  const theme = useTheme();

  const cssVars = {
    ...navSectionCssVars.horizontal(theme),
    ...overridesVars,
  };

  return (
    <Scrollbar
      sx={{ height: 1 }}
      slotProps={{
        content: { height: 1, display: 'flex', alignItems: 'center' },
      }}
    >
      <Stack
        component="nav"
        direction="row"
        alignItems="center"
        className={navSectionClasses.horizontal.root}
        sx={{
          ...cssVars,
          mx: 'auto',
          height: 1,
          minHeight: 'var(--nav-height)',
          ...sx,
        }}
      >
        <NavUl sx={{ flexDirection: 'row', gap: 'var(--nav-item-gap)' }}>
          {data.map((group) => (
            <Group
              key={group.subheader ?? group.items[0].title}
              render={render}
              cssVars={cssVars}
              items={group.items}
              slotProps={slotProps}
              enabledRootRedirect={enabledRootRedirect}
            />
          ))}
        </NavUl>
      </Stack>
    </Scrollbar>
  );
}

function Group({ items, render, slotProps, enabledRootRedirect, cssVars }: NavGroupProps) {
  return (
    <NavLi>
      <NavUl sx={{ flexDirection: 'row', gap: 'var(--nav-item-gap)' }}>
        {items.map((list) => (
          <NavList
            key={list.title}
            depth={1}
            data={list}
            render={render}
            cssVars={cssVars}
            slotProps={slotProps}
            enabledRootRedirect={enabledRootRedirect}
          />
        ))}
      </NavUl>
    </NavLi>
  );
}
