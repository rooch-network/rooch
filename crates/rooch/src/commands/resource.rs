use move_core_types::{
    account_address::AccountAddress,
    language_storage::{TypeTag},
    parser::parse_type_tag,
};
use moveos_types::{
    move_types::{StructId},
};
use moveos_client::Client;


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

impl ResourceCommand {
    pub async fn execute(self) -> anyhow::Result<()> {
        let resp = self
            .client
            .resource(
                self.address,
                self.resource.module_id.clone(),
                self.resource.struct_id.clone(),
                self.type_args,
            )
            .await?;
        println!("{:?}", resp);
        Ok(())
    }
}
