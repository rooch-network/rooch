// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::log::FrameName;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::IdentStr;
use move_core_types::language_storage::{ModuleId, TypeTag};
use std::fmt;
use std::fmt::Display;

/// Wrapper to help render the underlying data in human readable formats that are
/// desirable for textual outputs and flamegraphs.
pub(crate) struct Render<'a, T>(pub &'a T);

impl<'a> Display for Render<'a, AccountAddress> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addr_short = self.0.short_str_lossless();
        write!(f, "0x")?;
        if addr_short.len() > 4 {
            write!(f, "{}..", &addr_short[..4])
        } else {
            write!(f, "{}", addr_short)
        }
    }
}

impl<'a> Display for Render<'a, ModuleId> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}", Render(self.0.address()), self.0.name())
    }
}

impl<'a> Display for Render<'a, (&'a ModuleId, &'a IdentStr, &'a [TypeTag])> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}", Render(self.0 .0), self.0 .1)?;
        if !self.0 .2.is_empty() {
            write!(
                f,
                "<{}>",
                self.0
                     .2
                    .iter()
                    .map(|ty| format!("{}", ty))
                    .collect::<Vec<_>>()
                    .join(",")
            )?;
        }
        Ok(())
    }
}

impl Display for FrameName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Script => write!(f, "<script>"),
            Self::Function {
                module_id,
                name: fn_name,
                ty_args,
            } => write!(
                f,
                "{}",
                Render(&(module_id, fn_name.as_ident_str(), ty_args.as_slice())),
            ),
        }
    }
}
