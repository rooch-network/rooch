use async_trait::async_trait;
use move_core_types::{account_address::AccountAddress, language_storage::StructTag};
use rooch_client::Client;
use rooch_server::jsonrpc_types::AnnotatedMoveStructView;
use rooch_types::cli::{CliError, CliResult, CommandAction};

#[derive(clap::Parser)]
pub struct ResourceCommand {
    /// Account address where the resource stored.
    #[clap(long)]
    pub address: AccountAddress,

    /// Struct name as `<ADDRESS>::<MODULE_ID>::<STRUCT_NAME><TypeParam1?, TypeParam2?>`
    /// Example: `0x123::counter::Counter`, `0x123::counter::Box<0x123::counter::Counter>`
    #[clap(long = "resource")]
    pub resource: StructTag,

    /// RPC client options.
    #[clap(flatten)]
    client: Client,
}

#[async_trait]
impl CommandAction<Option<AnnotatedMoveStructView>> for ResourceCommand {
    async fn execute(self) -> CliResult<Option<AnnotatedMoveStructView>> {
        let resp = self
            .client
            .get_resource(self.address, self.resource)
            .await
            .map_err(CliError::from)?;
        Ok(resp)
    }
}
