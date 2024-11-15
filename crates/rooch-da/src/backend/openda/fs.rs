// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::operator::Operator;
use async_trait::async_trait;
use rooch_types::da::segment::SegmentID;

#[async_trait]
impl Operator for opendal::Operator {
    async fn submit_segment(
        &self,
        segment_id: SegmentID,
        segment_bytes: Vec<u8>,
        prefix: Option<String>,
    ) -> anyhow::Result<()> {
        let path = match prefix {
            Some(prefix) => format!("{}/{}", prefix, segment_id),
            None => segment_id.to_string(),
        };
        let mut w = self.writer(&path).await?;
        w.write(segment_bytes).await?;
        w.close().await?;
        Ok(())
    }
}
