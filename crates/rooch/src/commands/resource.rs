use async_trait::async_trait;
use move_core_types::{
    account_address::AccountAddress, language_storage::TypeTag, parser::parse_type_tag,
};
use moveos_types::move_types::StructId;
use rooch_client::Client;
use rooch_types::cli::{CliError, CliResult, CommandAction};

#[derive(clap::Parser)]
pub struct ResourceCommand {
    /// Account address where the resource stored.
    #[clap(long)]
    pub address: AccountAddress,

    /// Struct name as `<ADDRESS>::<MODULE_ID>::<STRUCT_NAME>`
    /// Example: `0x123::counter::Counter`
    #[clap(long)]
    pub resource: StructId,

    /// TypeTag arguments separated by spaces.
    /// Example: `u8 u16 u32 u64 u128 u256 bool address`
    #[clap(
            long = "type-args",
            parse(try_from_str = parse_type_tag),
            takes_value(true),
            multiple_values(true),
            multiple_occurrences(true)
        )]
    pub type_args: Vec<TypeTag>,

    /// RPC client options.
    #[clap(flatten)]
    client: Client,
}

#[async_trait]
impl CommandAction<Option<String>> for ResourceCommand {
    async fn execute(self) -> CliResult<Option<String>> {
        let resp = self
            .client
            .resource(
                self.address,
                self.resource.module_id.clone(),
                self.resource.struct_id.clone(),
                self.type_args,
            )
            .await
            .map_err(CliError::from)?;
        Ok(resp)
    }
}
