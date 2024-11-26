// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
// Author: Jason Jo

import { createTheme } from "@mui/material";

export const theme = createTheme({
  palette: {
    primary: {
      main: "#0F172A",
    },
  },
  typography: {
    fontFamily: [
      "Raleway Variable",
      "Montserrat",
      "ui-sans-serif",
      "system-ui",
      "sans-serif",
      "Apple Color Emoji",
      "Segoe UI Emoji",
      "Segoe UI Symbol",
      "Noto Color Emoji",
    ].join(","),
    allVariants: {
      fontFamily: "inherit",
      fontSize: undefined,
      fontWeight: undefined,
      textTransform: "unset",
      margin: undefined,
    },
  },
  shape: {
    borderRadius: 12,
  },
  components: {
    MuiStack: {
      defaultProps: {
        direction: "row",
        alignItems: "center",
      },
    },
    MuiChip: {
      defaultProps: {
        sx: {
          borderRadius: "12px",
        },
      },
    },
    MuiButton: {
      defaultProps: {
        sx: {
          boxShadow: "none",
        },
      },
    },
  },
});
