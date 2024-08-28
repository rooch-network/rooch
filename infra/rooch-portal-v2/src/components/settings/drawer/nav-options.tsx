import type { ButtonBaseProps } from '@mui/material/ButtonBase';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import { useTheme } from '@mui/material/styles';
import ButtonBase from '@mui/material/ButtonBase';

import { CONFIG } from 'src/config-global';
import { varAlpha, stylesMode } from 'src/theme/styles';

import { Block } from './styles';
import { SvgColor, svgColorClasses } from '../../svg-color';

import type { SettingsState } from '../types';

type Props = {
  value: {
    color: SettingsState['navColor'];
    layout: SettingsState['navLayout'];
  };
  options: {
    colors: SettingsState['navColor'][];
    layouts: SettingsState['navLayout'][];
  };
  onClickOption: {
    color: (newValue: SettingsState['navColor']) => void;
    layout: (newValue: SettingsState['navLayout']) => void;
  };
  hideNavColor?: boolean;
  hideNavLayout?: boolean;
};

export function NavOptions({ options, value, onClickOption, hideNavColor, hideNavLayout }: Props) {
  const theme = useTheme();

  const cssVars = {
    '--item-radius': '12px',
    '--item-bg': theme.vars.palette.grey[500],
    '--item-border-color': varAlpha(theme.vars.palette.grey['500Channel'], 0.08),
    '--item-active-color': `linear-gradient(135deg, ${theme.vars.palette.primary.light} 0%, ${theme.vars.palette.primary.main} 100%)`,
    '--item-active-shadow-light': `-8px 8px 20px -4px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.12)}`,
    '--item-active-shadow-dark': `-8px 8px 20px -4px ${varAlpha(theme.vars.palette.common.blackChannel, 0.12)}`,
  };

  const labelStyles: React.CSSProperties = {
    display: 'block',
    lineHeight: '14px',
    color: 'text.secondary',
    fontWeight: 'fontWeightSemiBold',
    fontSize: theme.typography.pxToRem(11),
  };

  const renderLayout = (
    <div>
      <Box component="span" sx={labelStyles}>
        Layout
      </Box>
      <Box gap={1.5} display="flex" sx={{ mt: 1.5 }}>
        {options.layouts.map((option) => (
          <LayoutOption
            key={option}
            option={option}
            selected={value.layout === option}
            onClick={() => onClickOption.layout(option)}
          />
        ))}
      </Box>
    </div>
  );

  const renderColor = (
    <div>
      <Box component="span" sx={labelStyles}>
        Color
      </Box>
      <Box gap={1.5} display="flex" sx={{ mt: 1.5 }}>
        {options.colors.map((option) => (
          <ColorOption
            key={option}
            option={option}
            selected={value.color === option}
            onClick={() => onClickOption.color(option)}
          />
        ))}
      </Box>
    </div>
  );

  return (
    <Block title="Nav" tooltip="Dashboard only" sx={{ ...cssVars, gap: 2.5 }}>
      {!hideNavLayout && renderLayout}
      {!hideNavColor && renderColor}
    </Block>
  );
}

type OptionProps = ButtonBaseProps & {
  option: string;
  selected: boolean;
};

export function LayoutOption({ option, selected, sx, ...other }: OptionProps) {
  const renderNav = () => {
    const baseStyles = { flexShrink: 0, borderRadius: 1, bgcolor: 'var(--item-bg)' };

    const circle = (
      <Box
        sx={{
          ...baseStyles,
          width: 10,
          height: 10,
          opacity: 0.8,
          ...(selected && { opacity: 1, background: 'var(--item-active-color)' }),
        }}
      />
    );

    const primaryItem = (
      <Box
        sx={{
          ...baseStyles,
          width: 1,
          height: 4,
          opacity: 0.48,
          ...(option === 'horizontal' && { width: 16 }),
          ...(selected && { background: 'var(--item-active-color)' }),
        }}
      />
    );

    const secondaryItem = (
      <Box
        sx={{
          ...baseStyles,
          width: 1,
          height: 4,
          maxWidth: 14,
          opacity: 0.24,
          ...(option === 'horizontal' && { maxWidth: 10 }),
          ...(selected && { background: 'var(--item-active-color)' }),
        }}
      />
    );

    return (
      <Stack
        spacing={0.5}
        flexShrink={0}
        sx={{
          p: 0.75,
          width: 32,
          height: 1,
          borderRightWidth: 1,
          borderRightStyle: 'solid',
          borderRightColor: 'var(--item-border-color)',
          ...(option === 'mini' && {
            width: 22,
          }),
          ...(option === 'horizontal' && {
            width: 1,
            height: 22,
            borderRight: 'none',
            alignItems: 'center',
            flexDirection: 'row',
            borderBottomWidth: 1,
            borderBottomStyle: 'solid',
            borderBottomColor: 'var(--item-border-color)',
          }),
        }}
      >
        {circle}
        {primaryItem}
        {secondaryItem}
      </Stack>
    );
  };

  const renderContent = (
    <Box sx={{ p: 0.5, width: 1, height: 1, flexGrow: 1 }}>
      <Box
        sx={{
          width: 1,
          height: 1,
          opacity: 0.2,
          borderRadius: 0.75,
          bgcolor: 'var(--item-bg)',
          ...(selected && { background: 'var(--item-active-color)' }),
        }}
      />
    </Box>
  );

  return (
    <ButtonBase
      disableRipple
      sx={{
        width: 1,
        height: 64,
        borderWidth: 1,
        borderStyle: 'solid',
        borderRadius: 'var(--item-radius)',
        borderColor: 'var(--item-border-color)',
        ...(option === 'horizontal' && { flexDirection: 'column' }),
        ...(selected && {
          boxShadow: 'var(--item-active-shadow-light)',
          [stylesMode.dark]: {
            boxShadow: 'var(--item-active-shadow-dark)',
          },
        }),
        ...sx,
      }}
      {...other}
    >
      {renderNav()}
      {renderContent}
    </ButtonBase>
  );
}

export function ColorOption({ option, selected, sx, ...other }: OptionProps) {
  return (
    <ButtonBase
      disableRipple
      sx={{
        gap: 1.5,
        width: 1,
        height: 56,
        color: 'text.disabled',
        borderRadius: 'var(--item-radius)',
        ...(selected && {
          borderWidth: 1,
          borderStyle: 'solid',
          color: 'text.primary',
          borderColor: 'var(--item-border-color)',
          boxShadow: 'var(--item-active-shadow-light)',
          [`& .${svgColorClasses.root}`]: {
            background: 'var(--item-active-color)',
          },
          [stylesMode.dark]: {
            boxShadow: 'var(--item-active-shadow-dark)',
          },
        }),
        ...sx,
      }}
      {...other}
    >
      <SvgColor
        src={`${CONFIG.site.basePath}/assets/icons/setting/ic-sidebar-${option === 'integrate' ? 'outline' : 'filled'}.svg`}
      />

      <Box
        component="span"
        sx={{
          lineHeight: '18px',
          textTransform: 'capitalize',
          fontWeight: 'fontWeightSemiBold',
          fontSize: (theme) => theme.typography.pxToRem(13),
        }}
      >
        {option}
      </Box>
    </ButtonBase>
  );
}
