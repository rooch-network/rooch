use crate::types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use moveos_types::object::ObjectID;
use rooch_server::jsonrpc_types::AnnotatedObjectView;
use rooch_types::error::RoochResult;

#[derive(Debug, clap::Parser)]
pub struct ObjectCommand {
    /// Object id.
    #[clap(long)]
    pub id: ObjectID,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Option<AnnotatedObjectView>> for ObjectCommand {
    async fn execute(self) -> RoochResult<Option<AnnotatedObjectView>> {
        let client = self.context_options.build().await?.get_client().await?;
        let resp = client.get_object(self.id).await?;

        Ok(resp)
    }
}
