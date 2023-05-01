use anyhow::{anyhow, Result};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    parser::parse_type_tag,
};
use moveos_client::Client;
use std::str::FromStr;

/// Identifier of a module function
#[derive(Debug, Clone)]
pub struct StructId {
    pub module_id: ModuleId,
    pub struct_id: Identifier,
}

fn parse_function_id(function_id: &str) -> Result<StructId> {
    let ids: Vec<&str> = function_id.split_terminator("::").collect();
    if ids.len() != 3 {
        return Err(anyhow!(
            "StructId is not well formed.  Must be of the form <address>::<module>::<function>"
        ));
    }
    let address = AccountAddress::from_str(ids.first().unwrap())
        .map_err(|err| anyhow!("Module address error: {:?}", err.to_string()))?;
    let module = Identifier::from_str(ids.get(1).unwrap())
        .map_err(|err| anyhow!("Module name error: {:?}", err.to_string()))?;
    let struct_id = Identifier::from_str(ids.get(2).unwrap())
        .map_err(|err| anyhow!("Function name error: {:?}", err.to_string()))?;
    let module_id = ModuleId::new(address, module);
    Ok(StructId {
        module_id,
        struct_id,
    })
}

impl FromStr for StructId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_function_id(s)
    }
}

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
