use crate::types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use move_core_types::language_storage::StructTag;
use rooch_server::jsonrpc_types::AnnotatedEventView;
use rooch_types::error::{RoochError, RoochResult};

#[derive(clap::Parser)]
pub struct EventCommand {
    #[clap(subcommand)]
    cmd: EventSubCommand,
}

#[async_trait]
impl CommandAction<String> for EventCommand {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            EventSubCommand::GetEventsByEventHandle(cmd) => cmd.execute_serialized().await,
        }
    }
}

#[derive(clap::Subcommand)]
pub enum EventSubCommand {
    GetEventsByEventHandle(GetEventsByEventHandle),
}

#[derive(Debug, clap::Parser)]
pub struct GetEventsByEventHandle {
    #[clap(long = "event_handle_type")]
    event_handle_type: StructTag,
    #[clap(long)]
    cursor: Option<u64>,
    #[clap(long)]
    limit: Option<u64>,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Vec<Option<AnnotatedEventView>>> for GetEventsByEventHandle {
    async fn execute(self) -> RoochResult<Vec<Option<AnnotatedEventView>>> {
        let client = self.context_options.build().await?.get_client().await?;
        let resp = client
            .get_events_by_event_handle(self.event_handle_type.into(), self.cursor, self.limit)
            .await
            .map_err(RoochError::from)?;
        Ok(resp)
    }
}
