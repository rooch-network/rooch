use move_core_types::account_address::AccountAddress;
use moveos_client::Client;
use moveos_types::object::ObjectID;

#[derive(clap::Parser)]
pub struct ObjectCommand {
    /// Account address where the resource stored.
    #[clap(long)]
    pub id: AccountAddress,

    /// RPC client options.
    #[clap(flatten)]
    client: Client,
}

impl ObjectCommand {
    pub async fn execute(self) -> anyhow::Result<()> {
        let object_id = ObjectID::from(self.id);
        let resp = self.client.object(object_id).await?;
        println!("{:?}", resp);
        Ok(())
    }
}
