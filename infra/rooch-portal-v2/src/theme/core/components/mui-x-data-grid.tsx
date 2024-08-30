import type { SvgIconProps } from '@mui/material/SvgIcon';
import type { Theme, Components } from '@mui/material/styles';

import { listClasses } from '@mui/material/List';
import { paperClasses } from '@mui/material/Paper';
import { textFieldClasses } from '@mui/material/TextField';
import { inputBaseClasses } from '@mui/material/InputBase';
import { inputLabelClasses } from '@mui/material/InputLabel';
import { iconButtonClasses } from '@mui/material/IconButton';
import SvgIcon, { svgIconClasses } from '@mui/material/SvgIcon';
import { listItemIconClasses } from '@mui/material/ListItemIcon';
import { circularProgressClasses } from '@mui/material/CircularProgress';
import { formControlLabelClasses } from '@mui/material/FormControlLabel';

import { paper, varAlpha } from '../../styles';

const MuiDataGrid: Components<Theme>['MuiDataGrid'] = {
  defaultProps: {
    slots: {
      /* Column */
      columnSortedAscendingIcon: (props: SvgIconProps) => (
        <DataGridArrowUpIcon sx={{ color: 'text.primary' }} {...props} />
      ),
      columnSortedDescendingIcon: (props: SvgIconProps) => (
        <DataGridArrowDownIcon sx={{ color: 'text.primary' }} {...props} />
      ),
      columnUnsortedIcon: (props: SvgIconProps) => (
        <DataGridArrowUpIcon
          fontSize={props.fontSize}
          className={props.className}
          sx={{ color: 'text.disabled' }}
        />
      ),
      columnMenuIcon: (props: SvgIconProps) => <DataGridMoreIcon width={20} {...props} />,
      columnMenuSortAscendingIcon: (props: SvgIconProps) => <DataGridArrowUpIcon {...props} />,
      columnMenuSortDescendingIcon: (props: SvgIconProps) => <DataGridArrowDownIcon {...props} />,
      columnMenuFilterIcon: (props: SvgIconProps) => <DataGridFilterIcon {...props} />,
      columnMenuHideIcon: (props: SvgIconProps) => <DataGridEyeCloseIcon {...props} />,
      columnMenuManageColumnsIcon: (props: SvgIconProps) => <DataGridEyeIcon {...props} />,
      columnSelectorIcon: (props: SvgIconProps) => <DataGridEyeIcon {...props} />,
      /* Filter */
      filterPanelDeleteIcon: (props: SvgIconProps) => <DataGridCloseIcon {...props} />,
      openFilterButtonIcon: (props: SvgIconProps) => <DataGridFilterIcon {...props} />,
      columnFilteredIcon: (props: SvgIconProps) => (
        <DataGridFilterIcon sx={{ width: 16, color: 'text.primary' }} {...props} />
      ),
      /* Density */
      densityCompactIcon: (props: SvgIconProps) => <DataGridDensityCompactIcon {...props} />,
      densityStandardIcon: (props: SvgIconProps) => <DataGridDensityStandardIcon {...props} />,
      densityComfortableIcon: (props: SvgIconProps) => (
        <DataGridDensityComfortableIcon {...props} />
      ),
      /* Export */
      exportIcon: (props: SvgIconProps) => <DataGridExportIcon {...props} />,
      /*  Quick Filter */
      quickFilterIcon: (props: SvgIconProps) => (
        <DataGridSearchIcon sx={{ width: 24, height: 24, color: 'text.secondary' }} {...props} />
      ),
      quickFilterClearIcon: (props: SvgIconProps) => <DataGridCloseIcon {...props} />,
    },
    slotProps: {
      basePopper: { placement: 'bottom-end' },
      baseChip: { size: 'small' },
      baseSwitch: { size: 'small' },
      baseCheckbox: { size: 'small', disableRipple: true },
      baseInputLabel: { shrink: true },
      baseTextField: { variant: 'outlined' },
      baseSelect: { native: true, variant: 'outlined' },
    },
  },

  styleOverrides: {
    root: ({ theme }) => ({
      '--unstable_DataGrid-radius': 0,
      '--DataGrid-rowBorderColor': theme.vars.palette.divider,
      '--DataGrid-containerBackground': theme.vars.palette.background.neutral,
      '--unstable_DataGrid-headWeight': theme.typography.fontWeightSemiBold,
      borderWidth: 0,
      scrollbarWidth: 'thin',
      scrollbarColor: `${varAlpha(theme.vars.palette.text.disabledChannel, 0.4)} ${varAlpha(theme.vars.palette.text.disabledChannel, 0.08)}`,
      '& .MuiDataGrid-filler > div': { borderTopStyle: 'dashed' },
      '& .MuiDataGrid-topContainer::after': { height: 0 },
    }),
    withBorderColor: { borderColor: 'var(--DataGrid-rowBorderColor)' },
    /**
     * Column
     */
    columnHeader: ({ theme }) => ({
      fontSize: 14,
      color: theme.vars.palette.text.secondary,
      '&--sorted': { color: theme.vars.palette.text.primary },
    }),
    columnSeparator: { color: 'var(--DataGrid-rowBorderColor)' },
    /**
     * Row, Cell
     */
    cell: ({ theme }) => ({
      borderTopStyle: 'dashed',
      '&--editing': {
        boxShadow: 'none',
        backgroundColor: varAlpha(theme.vars.palette.primary.mainChannel, 0.08),
      },
    }),
    /**
     * Toolbar
     */
    toolbarContainer: ({ theme }) => ({
      gap: theme.spacing(2),
      padding: theme.spacing(2),
      [`& .${textFieldClasses.root}`]: {
        padding: 0,
        width: '100%',
        [`& .${inputBaseClasses.input}`]: {
          paddingTop: theme.spacing(2),
          paddingBottom: theme.spacing(2),
        },
        [theme.breakpoints.up('md')]: { width: 'unset' },
      },
    }),
    /**
     * Paper
     */
    paper: ({ theme }) => ({
      ...paper({ theme, dropdown: true }),
      padding: 0,
    }),
    menu: ({ theme }) => ({
      [`& .${paperClasses.root}`]: {
        ...paper({ theme, dropdown: true }),
        minWidth: 140,
      },
      [`& .${listClasses.root}`]: {
        padding: 0,
        [`& .${listItemIconClasses.root}`]: {
          minWidth: 0,
          marginRight: theme.spacing(2),
        },
      },
    }),
    /**
     * Icons
     */
    menuIcon: ({ theme }) => ({
      [`& .${iconButtonClasses.root}`]: {
        margin: theme.spacing(0, 1),
        padding: theme.spacing(0.25),
      },
    }),
    iconButtonContainer: ({ theme }) => ({
      [`& .${iconButtonClasses.root}`]: {
        padding: theme.spacing(0.25),
        marginLeft: theme.spacing(1),
      },
    }),
    /**
     * Footer
     */
    footerContainer: { minHeight: 'auto', borderTopStyle: 'dashed' },
    selectedRowCount: { display: 'none', whiteSpace: 'nowrap' },
    overlay: ({ theme }) => ({
      [`& .${circularProgressClasses.root}`]: { color: theme.vars.palette.text.primary },
    }),
    /**
     * Column panel
     */
    columnsManagementHeader: ({ theme }) => ({
      padding: theme.spacing(2.5, 2, 0, 2),
      [`& .${inputBaseClasses.input}`]: {
        paddingTop: theme.spacing(2),
        paddingBottom: theme.spacing(2),
      },
    }),
    columnsManagement: ({ theme }) => ({
      gap: theme.spacing(0.5),
      padding: theme.spacing(2, 1.5),
      [`& .${formControlLabelClasses.root}`]: { gap: 4, marginLeft: 0 },
    }),
    columnsManagementFooter: ({ theme }) => ({
      borderTopStyle: 'dashed',
      padding: theme.spacing(1.5),
      [`& .${formControlLabelClasses.root}`]: { gap: 4, marginLeft: 0 },
    }),
    /**
     * Filter panel
     */
    filterForm: ({ theme }) => ({
      alignItems: 'center',
      gap: theme.spacing(1.5),
      padding: theme.spacing(2),
      /* Fix label with input variant === 'outlined' */
      [`& .${inputLabelClasses.shrink}`]: { transform: 'translate(14px, -9px) scale(0.75)' },
    }),
    filterFormDeleteIcon: ({ theme }) => ({
      [`& .${iconButtonClasses.root}`]: {
        padding: theme.spacing(0.25),
        backgroundColor: varAlpha(theme.vars.palette.grey['500Channel'], 0.16),
        [`& .${svgIconClasses.root}`]: { width: 16, height: 16 },
      },
    }),
  },
};

export const dataGrid = { MuiDataGrid };

/**
 * Icons
 */
export const DataGridArrowUpIcon = ({ ...props }: SvgIconProps) => (
  <SvgIcon sx={{ width: 20, height: 20, ...props.sx }} {...props}>
    <path
      fill="currentColor"
      d="m8.303 11.596l3.327-3.431a.499.499 0 0 1 .74 0l6.43 6.63c.401.414.158 1.205-.37 1.205h-5.723z"
    />
    <path
      fill="currentColor"
      d="M11.293 16H5.57c-.528 0-.771-.791-.37-1.205l2.406-2.482z"
      opacity="0.5"
    />
  </SvgIcon>
);

export const DataGridArrowDownIcon = ({ ...props }: SvgIconProps) => (
  <SvgIcon sx={{ width: 20, height: 20, ...props.sx }} {...props}>
    <path
      fill="currentColor"
      d="m8.303 12.404l3.327 3.431c.213.22.527.22.74 0l6.43-6.63C19.201 8.79 18.958 8 18.43 8h-5.723z"
    />
    <path
      fill="currentColor"
      d="M11.293 8H5.57c-.528 0-.771.79-.37 1.205l2.406 2.481z"
      opacity="0.5"
    />
  </SvgIcon>
);

export const DataGridFilterIcon = ({ ...props }: SvgIconProps) => (
  <SvgIcon sx={{ width: 20, height: 20, ...props.sx }} {...props}>
    <path
      fill="currentColor"
      d="M19 3H5c-1.414 0-2.121 0-2.56.412C2 3.824 2 4.488 2 5.815v.69c0 1.037 0 1.556.26 1.986c.26.43.733.698 1.682 1.232l2.913 1.64c.636.358.955.537 1.183.735c.474.411.766.895.898 1.49c.064.284.064.618.064 1.285v2.67c0 .909 0 1.364.252 1.718c.252.355.7.53 1.594.88c1.879.734 2.818 1.101 3.486.683c.668-.417.668-1.372.668-3.282v-2.67c0-.666 0-1 .064-1.285a2.68 2.68 0 0 1 .899-1.49c.227-.197.546-.376 1.182-.735l2.913-1.64c.948-.533 1.423-.8 1.682-1.23c.26-.43.26-.95.26-1.988v-.69c0-1.326 0-1.99-.44-2.402C21.122 3 20.415 3 19 3"
    />
  </SvgIcon>
);

export const DataGridExportIcon = ({ ...props }: SvgIconProps) => (
  <SvgIcon sx={{ width: 20, height: 20, ...props.sx }} {...props}>
    <path
      fill="currentColor"
      fillRule="evenodd"
      d="M8.845 7.905a.75.75 0 0 0 1.06 0l1.72-1.72v8.19a.75.75 0 0 0 1.5 0v-8.19l1.72 1.72a.75.75 0 1 0 1.06-1.06l-3-3a.75.75 0 0 0-1.06 0l-3 3a.75.75 0 0 0 0 1.06"
      clipRule="evenodd"
    />
    <path
      fill="currentColor"
      d="M12.375 20.375a8 8 0 0 0 8-8h-3.75c-.943 0-1.414 0-1.707.293c-.293.293-.293.764-.293 1.707a2.25 2.25 0 0 1-4.5 0c0-.943 0-1.414-.293-1.707c-.293-.293-.764-.293-1.707-.293h-3.75a8 8 0 0 0 8 8"
    />
  </SvgIcon>
);

/* https://icon-sets.iconify.design/solar/eye-bold */
export const DataGridEyeIcon = ({ ...props }: SvgIconProps) => (
  <SvgIcon sx={{ width: 20, height: 20, ...props.sx }} {...props}>
    <path fill="currentColor" d="M9.75 12a2.25 2.25 0 1 1 4.5 0a2.25 2.25 0 0 1-4.5 0" />
    <path
      fill="currentColor"
      fillRule="evenodd"
      d="M2 12c0 1.64.425 2.191 1.275 3.296C4.972 17.5 7.818 20 12 20c4.182 0 7.028-2.5 8.725-4.704C21.575 14.192 22 13.639 22 12c0-1.64-.425-2.191-1.275-3.296C19.028 6.5 16.182 4 12 4C7.818 4 4.972 6.5 3.275 8.704C2.425 9.81 2 10.361 2 12m10-3.75a3.75 3.75 0 1 0 0 7.5a3.75 3.75 0 0 0 0-7.5"
      clipRule="evenodd"
    />
  </SvgIcon>
);

/* https://icon-sets.iconify.design/ph/eye-closed-bold */
export const DataGridEyeCloseIcon = ({ ...props }: SvgIconProps) => (
  <SvgIcon sx={{ width: 20, height: 20, ...props.sx }} {...props}>
    <path
      fill="currentColor"
      fillRule="evenodd"
      d="M1.606 6.08a1 1 0 0 1 1.313.526L2 7l.92-.394v-.001c0-.001 0 0 0 0l.003.009l.021.045c.02.042.051.108.094.194c.086.172.219.424.4.729a13.37 13.37 0 0 0 1.67 2.237a11.966 11.966 0 0 0 .59.592C7.18 11.8 9.251 13 12 13a8.706 8.706 0 0 0 3.22-.602c1.227-.483 2.254-1.21 3.096-1.998a13.053 13.053 0 0 0 2.733-3.725l.027-.058l.005-.011a1 1 0 0 1 1.838.788L22 7l.92.394l-.003.005l-.004.008l-.011.026l-.04.087a14.045 14.045 0 0 1-.741 1.348a15.368 15.368 0 0 1-1.711 2.256l.797.797a1 1 0 0 1-1.414 1.415l-.84-.84a11.81 11.81 0 0 1-1.897 1.256l.782 1.202a1 1 0 1 1-1.676 1.091l-.986-1.514c-.679.208-1.404.355-2.176.424V16.5a1 1 0 0 1-2 0v-1.544c-.775-.07-1.5-.217-2.177-.425l-.985 1.514a1 1 0 0 1-1.676-1.09l.782-1.203c-.7-.37-1.332-.8-1.897-1.257l-.84.84a1 1 0 0 1-1.414-1.414l.797-.797a15.406 15.406 0 0 1-1.87-2.519a13.457 13.457 0 0 1-.591-1.107a5.418 5.418 0 0 1-.033-.072l-.01-.021l-.002-.007l-.001-.002v-.001C1.08 7.395 1.08 7.394 2 7l-.919.395a1 1 0 0 1 .525-1.314"
      clipRule="evenodd"
    />
  </SvgIcon>
);

/* https://icon-sets.iconify.design/eva/search-fill */
export const DataGridSearchIcon = ({ ...props }: SvgIconProps) => (
  <SvgIcon sx={{ width: 20, height: 20, ...props.sx }} {...props}>
    <path
      fill="currentColor"
      d="m20.71 19.29l-3.4-3.39A7.92 7.92 0 0 0 19 11a8 8 0 1 0-8 8a7.92 7.92 0 0 0 4.9-1.69l3.39 3.4a1 1 0 0 0 1.42 0a1 1 0 0 0 0-1.42M5 11a6 6 0 1 1 6 6a6 6 0 0 1-6-6"
    />
  </SvgIcon>
);

/* https://icon-sets.iconify.design/eva/close-fill */
export const DataGridCloseIcon = ({ ...props }: SvgIconProps) => (
  <SvgIcon sx={{ width: 20, height: 20, ...props.sx }} {...props}>
    <path
      fill="currentColor"
      d="m13.41 12l4.3-4.29a1 1 0 1 0-1.42-1.42L12 10.59l-4.29-4.3a1 1 0 0 0-1.42 1.42l4.3 4.29l-4.3 4.29a1 1 0 0 0 0 1.42a1 1 0 0 0 1.42 0l4.29-4.3l4.29 4.3a1 1 0 0 0 1.42 0a1 1 0 0 0 0-1.42Z"
    />
  </SvgIcon>
);

/* https://icon-sets.iconify.design/mingcute/more-1-fill */
export const DataGridMoreIcon = ({ ...props }: SvgIconProps) => (
  <SvgIcon sx={{ width: 20, height: 20, ...props.sx }} {...props}>
    <g fill="none">
      <path d="M24 0v24H0V0zM12.593 23.258l-.011.002l-.071.035l-.02.004l-.014-.004l-.071-.035c-.01-.004-.019-.001-.024.005l-.004.01l-.017.428l.005.02l.01.013l.104.074l.015.004l.012-.004l.104-.074l.012-.016l.004-.017l-.017-.427c-.002-.01-.009-.017-.017-.018m.265-.113l-.013.002l-.185.093l-.01.01l-.003.011l.018.43l.005.012l.008.007l.201.093c.012.004.023 0 .029-.008l.004-.014l-.034-.614c-.003-.012-.01-.02-.02-.022m-.715.002a.023.023 0 0 0-.027.006l-.006.014l-.034.614c0 .012.007.02.017.024l.015-.002l.201-.093l.01-.008l.004-.011l.017-.43l-.003-.012l-.01-.01z" />
      <path
        fill="currentColor"
        d="M5 10a2 2 0 1 1 0 4a2 2 0 0 1 0-4m7 0a2 2 0 1 1 0 4a2 2 0 0 1 0-4m7 0a2 2 0 1 1 0 4a2 2 0 0 1 0-4"
      />
    </g>
  </SvgIcon>
);

/* https://icon-sets.iconify.design/material-symbols/table-rows-narrow-rounded */
export const DataGridDensityCompactIcon = ({ ...props }: SvgIconProps) => (
  <SvgIcon sx={{ width: 20, height: 20, ...props.sx }} {...props}>
    <path
      fill="currentColor"
      d="M4 15.5q-.425 0-.712-.288T3 14.5V14q0-.425.288-.712T4 13h16q.425 0 .713.288T21 14v.5q0 .425-.288.713T20 15.5zM4 11q-.425 0-.712-.288T3 10v-.5q0-.425.288-.712T4 8.5h16q.425 0 .713.288T21 9.5v.5q0 .425-.288.713T20 11zm0-4.5q-.425 0-.712-.288T3 5.5V5q0-.425.288-.712T4 4h16q.425 0 .713.288T21 5v.5q0 .425-.288.713T20 6.5zM4 20q-.425 0-.712-.288T3 19v-.5q0-.425.288-.712T4 17.5h16q.425 0 .713.288T21 18.5v.5q0 .425-.288.713T20 20z"
    />
  </SvgIcon>
);

/* https://icon-sets.iconify.design/mingcute/rows-2-fill */
export const DataGridDensityComfortableIcon = ({ ...props }: SvgIconProps) => (
  <SvgIcon sx={{ width: 20, height: 20, ...props.sx }} {...props}>
    <g fill="none" fillRule="evenodd">
      <path d="M24 0v24H0V0zM12.593 23.258l-.011.002l-.071.035l-.02.004l-.014-.004l-.071-.035c-.01-.004-.019-.001-.024.005l-.004.01l-.017.428l.005.02l.01.013l.104.074l.015.004l.012-.004l.104-.074l.012-.016l.004-.017l-.017-.427c-.002-.01-.009-.017-.017-.018m.265-.113l-.013.002l-.185.093l-.01.01l-.003.011l.018.43l.005.012l.008.007l.201.093c.012.004.023 0 .029-.008l.004-.014l-.034-.614c-.003-.012-.01-.02-.02-.022m-.715.002a.023.023 0 0 0-.027.006l-.006.014l-.034.614c0 .012.007.02.017.024l.015-.002l.201-.093l.01-.008l.004-.011l.017-.43l-.003-.012l-.01-.01z" />
      <path
        fill="currentColor"
        d="M5 3a2 2 0 0 0-2 2v6h18V5a2 2 0 0 0-2-2zm16 10H3v6a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2z"
      />
    </g>
  </SvgIcon>
);

/* https://icon-sets.iconify.design/mingcute/rows-4-fill */
export const DataGridDensityStandardIcon = ({ ...props }: SvgIconProps) => (
  <SvgIcon sx={{ width: 20, height: 20, ...props.sx }} {...props}>
    <g fill="none">
      <path d="M24 0v24H0V0zM12.593 23.258l-.011.002l-.071.035l-.02.004l-.014-.004l-.071-.035c-.01-.004-.019-.001-.024.005l-.004.01l-.017.428l.005.02l.01.013l.104.074l.015.004l.012-.004l.104-.074l.012-.016l.004-.017l-.017-.427c-.002-.01-.009-.017-.017-.018m.265-.113l-.013.002l-.185.093l-.01.01l-.003.011l.018.43l.005.012l.008.007l.201.093c.012.004.023 0 .029-.008l.004-.014l-.034-.614c-.003-.012-.01-.02-.02-.022m-.715.002a.023.023 0 0 0-.027.006l-.006.014l-.034.614c0 .012.007.02.017.024l.015-.002l.201-.093l.01-.008l.004-.011l.017-.43l-.003-.012l-.01-.01z" />
      <path
        fill="currentColor"
        d="M21 16v3a2 2 0 0 1-1.85 1.995L19 21H5a2 2 0 0 1-1.995-1.85L3 19v-3zm0-6v4H3v-4zm-2-7a2 2 0 0 1 2 2v3H3V5a2 2 0 0 1 2-2z"
      />
    </g>
  </SvgIcon>
);
