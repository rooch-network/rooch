// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, clap::ValueEnum, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceStatus {
    /// The service is active and running normally.
    #[default]
    Active,
    /// The service is in maintenance mode.
    Maintenance,
    /// The service is in read-only mode.
    ReadOnlyMode,
    /// The service is in date import mode.
    DateImportMode,
}

impl ServiceStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, ServiceStatus::Active)
    }

    pub fn is_maintenance(&self) -> bool {
        matches!(self, ServiceStatus::Maintenance)
    }

    pub fn is_read_only_mode(&self) -> bool {
        matches!(self, ServiceStatus::ReadOnlyMode)
    }

    pub fn is_date_import_mode(&self) -> bool {
        matches!(self, ServiceStatus::DateImportMode)
    }
}
