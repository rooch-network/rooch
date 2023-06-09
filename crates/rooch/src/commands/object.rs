use crate::types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use moveos_types::{access_path::AccessPath, object::ObjectID};
use rooch_server::jsonrpc_types::AnnotatedStateView;
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
impl CommandAction<Option<AnnotatedStateView>> for ObjectCommand {
    async fn execute(self) -> RoochResult<Option<AnnotatedStateView>> {
        let client = self.context_options.build().await?.get_client().await?;
        let resp = client
            .get_annotated_states(AccessPath::object(self.id))
            .await?
            .pop()
            .flatten();

        Ok(resp)
    }
}
