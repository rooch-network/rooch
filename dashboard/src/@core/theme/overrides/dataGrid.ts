// ** Type Import
import { OwnerStateThemeType } from './'

const DataGrid = () => {
  return {
    MuiDataGrid: {
      styleOverrides: {
        root: ({ theme }: OwnerStateThemeType) => ({
          border: 0,
          color: theme.palette.text.primary,
          '& .MuiDataGrid-columnHeader:focus, & .MuiDataGrid-columnHeader:focus-within': {
            outline: 'none'
          }
        }),
        toolbarContainer: ({ theme }: OwnerStateThemeType) => ({
          paddingRight: `${theme.spacing(5)} !important`,
          paddingLeft: `${theme.spacing(3.25)} !important`
        }),
        columnHeaders: ({ theme }: OwnerStateThemeType) => ({
          borderTop: `1px solid ${theme.palette.divider}`
        }),
        columnHeader: ({ theme }: OwnerStateThemeType) => ({
          '&:not(.MuiDataGrid-columnHeaderCheckbox)': {
            paddingLeft: theme.spacing(4),
            paddingRight: theme.spacing(4),
            '&:first-of-type': {
              paddingLeft: theme.spacing(5)
            }
          },
          '&:last-of-type': {
            paddingRight: theme.spacing(5)
          }
        }),
        columnHeaderCheckbox: {
          maxWidth: '58px !important',
          minWidth: '58px !important'
        },
        columnHeaderTitleContainer: {
          padding: 0
        },
        columnHeaderTitle: {
          fontWeight: 600,
          fontSize: '0.75rem',
          letterSpacing: '0.17px',
          textTransform: 'uppercase'
        },
        columnSeparator: ({ theme }: OwnerStateThemeType) => ({
          color: theme.palette.divider
        }),
        row: {
          '&:last-child': {
            '& .MuiDataGrid-cell': {
              borderBottom: 0
            }
          }
        },
        cell: ({ theme }: OwnerStateThemeType) => ({
          borderColor: theme.palette.divider,
          '&:not(.MuiDataGrid-cellCheckbox)': {
            paddingLeft: theme.spacing(4),
            paddingRight: theme.spacing(4),
            '&:first-of-type': {
              paddingLeft: theme.spacing(5)
            }
          },
          '&:last-of-type': {
            paddingRight: theme.spacing(5)
          },
          '&:focus, &:focus-within': {
            outline: 'none'
          }
        }),
        cellCheckbox: {
          maxWidth: '58px !important',
          minWidth: '58px !important'
        },
        editInputCell: ({ theme }: OwnerStateThemeType) => ({
          padding: 0,
          color: theme.palette.text.primary,
          '& .MuiInputBase-input': {
            padding: 0
          }
        }),
        footerContainer: ({ theme }: OwnerStateThemeType) => ({
          borderTop: `1px solid ${theme.palette.divider}`,
          '& .MuiTablePagination-toolbar': {
            paddingLeft: `${theme.spacing(4)} !important`,
            paddingRight: `${theme.spacing(4)} !important`
          },
          '& .MuiTablePagination-displayedRows, & .MuiTablePagination-selectLabel': {
            color: theme.palette.text.primary
          }
        }),
        selectedRowCount: ({ theme }: OwnerStateThemeType) => ({
          margin: 0,
          paddingLeft: theme.spacing(4),
          paddingRight: theme.spacing(4)
        })
      }
    }
  }
}

export default DataGrid
