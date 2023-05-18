use async_trait::async_trait;
use moveos_types::object::ObjectID;
use rooch_client::Client;
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
impl CommandAction<Option<String>> for ObjectCommand {
    async fn execute(self) -> CliResult<Option<String>> {
        let resp = self.client.object(self.id).await?;
        Ok(resp)
    }
}
