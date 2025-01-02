// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::adapter::OpenDAAdapter;
use async_trait::async_trait;
use rooch_types::da::segment::SegmentID;
use std::time::Duration;

pub(crate) const BACK_OFF_MIN_DELAY: Duration = Duration::from_millis(300);

#[async_trait]
impl OpenDAAdapter for opendal::Operator {
    async fn submit_segment(
        &self,
        segment_id: SegmentID,
        segment_bytes: &[u8],
    ) -> anyhow::Result<()> {
        let path = segment_id.to_string();
        let mut w = self.writer(&path).await?;
        w.write(segment_bytes.to_vec()).await?;
        w.close().await?;
        Ok(())
    }
}
