use crate::types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use move_core_types::{account_address::AccountAddress, language_storage::StructTag};
use rooch_server::jsonrpc_types::AnnotatedMoveStructView;
use rooch_types::error::{RoochError, RoochResult};

#[derive(Debug, clap::Parser)]
pub struct ResourceCommand {
    /// Account address where the resource stored.
    #[clap(long)]
    pub address: AccountAddress,

    /// Struct name as `<ADDRESS>::<MODULE_ID>::<STRUCT_NAME><TypeParam1?, TypeParam2?>`
    /// Example: `0x123::counter::Counter`, `0x123::counter::Box<0x123::counter::Counter>`
    #[clap(long = "resource")]
    pub resource: StructTag,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Option<AnnotatedMoveStructView>> for ResourceCommand {
    async fn execute(self) -> RoochResult<Option<AnnotatedMoveStructView>> {
        let client = self.context_options.build().await?.get_client().await?;
        let resp = client
            .get_resource(self.address, self.resource)
            .await
            .map_err(RoochError::from)?;
        Ok(resp)
    }
}
