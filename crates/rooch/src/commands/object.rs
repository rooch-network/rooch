use async_trait::async_trait;
use moveos_types::object::ObjectID;
use rooch_client::Client;
use rooch_server::jsonrpc_types::AnnotatedObjectView;
use rooch_types::cli::{CliResult, CommandAction};
#[derive(clap::Parser)]
pub struct ObjectCommand {
    /// Object id.
    #[clap(long)]
    pub id: ObjectID,

    /// RPC client options.
    #[clap(flatten)]
    client: Client,
}

#[async_trait]
impl CommandAction<Option<AnnotatedObjectView>> for ObjectCommand {
    async fn execute(self) -> CliResult<Option<AnnotatedObjectView>> {
        let resp = self.client.get_object(self.id).await?;
        Ok(resp)
    }
}
