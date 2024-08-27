import type { SvgIconProps } from '@mui/material/SvgIcon';
import type { Theme, Components } from '@mui/material/styles';

import { ratingClasses } from '@mui/material/Rating';
import SvgIcon, { svgIconClasses } from '@mui/material/SvgIcon';

import { varAlpha } from '../../styles';

/**
 * Icons
 */
export const RatingIcon = (props: SvgIconProps) => (
  <SvgIcon {...props}>
    <path d="M17.56,21 C17.4000767,21.0006435 17.2423316,20.9629218 17.1,20.89 L12,18.22 L6.9,20.89 C6.56213339,21.067663 6.15259539,21.0374771 5.8444287,20.8121966 C5.53626201,20.5869161 5.38323252,20.2058459 5.45,19.83 L6.45,14.2 L2.33,10.2 C2.06805623,9.93860108 1.9718844,9.55391377 2.08,9.2 C2.19824414,8.83742187 2.51242293,8.57366684 2.89,8.52 L8.59,7.69 L11.1,2.56 C11.2670864,2.21500967 11.6166774,1.99588989 12,1.99588989 C12.3833226,1.99588989 12.7329136,2.21500967 12.9,2.56 L15.44,7.68 L21.14,8.51 C21.5175771,8.56366684 21.8317559,8.82742187 21.95,9.19 C22.0581156,9.54391377 21.9619438,9.92860108 21.7,10.19 L17.58,14.19 L18.58,19.82 C18.652893,20.2027971 18.4967826,20.5930731 18.18,20.82 C17.9989179,20.9468967 17.7808835,21.010197 17.56,21 L17.56,21 Z" />
  </SvgIcon>
);

const MuiRating: Components<Theme>['MuiRating'] = {
  defaultProps: { emptyIcon: <RatingIcon />, icon: <RatingIcon /> },
  styleOverrides: {
    root: { [`&.${ratingClasses.disabled}`]: { opacity: 0.48 } },
    iconEmpty: ({ theme }) => ({ color: varAlpha(theme.vars.palette.grey['500Channel'], 0.48) }),
    sizeSmall: { [`& .${svgIconClasses.root}`]: { width: 20, height: 20 } },
    sizeMedium: { [`& .${svgIconClasses.root}`]: { width: 24, height: 24 } },
    sizeLarge: { [`& .${svgIconClasses.root}`]: { width: 28, height: 28 } },
  },
};

export const rating = { MuiRating };
