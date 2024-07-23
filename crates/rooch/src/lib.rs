// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::event::EventCommand;
use crate::commands::indexer::Indexer;
use crate::commands::statedb::Statedb;
use clap::builder::styling::{AnsiColor, Effects, Styles};
use cli_types::CommandAction;
use commands::{
    abi::ABI, account::Account, env::Env, genesis::Genesis, init::Init, move_cli::MoveCli,
    object::ObjectCommand, resource::ResourceCommand, rpc::Rpc, server::Server,
    session_key::SessionKey, state::StateCommand, transaction::Transaction,
};
use git2::{Oid, Repository};
use rooch_types::error::RoochResult;

pub mod cli_types;
pub mod commands;
pub mod utils;

#[derive(clap::Parser)]
#[clap(author, long_version(get_latest_tag_and_commit_hash().unwrap().commit_hash_str().unwrap()), about, long_about = None,
styles = Styles::styled()
.header(AnsiColor::Green.on_default() | Effects::BOLD)
.usage(AnsiColor::Green.on_default() | Effects::BOLD)
.literal(AnsiColor::Blue.on_default() | Effects::BOLD)
.placeholder(AnsiColor::Cyan.on_default()))]
pub struct RoochCli {
    #[clap(subcommand)]
    pub cmd: Command,
}

#[allow(clippy::large_enum_variant)]
#[derive(clap::Parser)]
pub enum Command {
    Account(Account),
    Init(Init),
    Move(MoveCli),
    Server(Server),
    State(StateCommand),
    Object(ObjectCommand),
    Resource(ResourceCommand),
    Transaction(Transaction),
    Event(EventCommand),
    ABI(ABI),
    Env(Env),
    SessionKey(SessionKey),
    Rpc(Rpc),
    Statedb(Statedb),
    Indexer(Indexer),
    Genesis(Genesis),
}

struct TagData<'a> {
    latest_tag_name: Option<String>,
    commit_hash_str: Option<String>,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> TagData<'a> {
    fn latest_tag_name(&self) -> Option<&str> {
        self.latest_tag_name.as_deref()
    }

    fn commit_hash_str(&self) -> Option<&str> {
        self.commit_hash_str.as_deref()
    }
}

fn get_latest_tag_and_commit_hash() -> Result<TagData<'static>, anyhow::Error> {
    // Open the repository
    let repo = Repository::open(".")?;

    // Get all tag names with a specific pattern
    let tag_names = repo.tag_names(Some("v*.*.*"))?;

    // Collect tags with their commit OIDs
    let mut tags_with_commits: Vec<(String, Oid)> = vec![];

    for name in tag_names.iter().flatten() {
        if let Ok(reference) = repo.find_reference(name) {
            if let Ok(tag) = reference.peel_to_tag() {
                let target_commit = tag
                    .target()
                    .map_err(|_| anyhow::anyhow!("Tag has no target commit"))?;
                tags_with_commits.push((name.to_string(), target_commit.id()));
            }
        }
    }

    // Sort tags by their commit OID in descending order
    tags_with_commits.sort_by(|a, b| b.1.cmp(&a.1));

    // Create a TagData struct to hold the results
    let tag_data =
        if let Some((latest_tag_name, commit_hash)) = tags_with_commits.into_iter().next() {
            let commit_hash_str = commit_hash.to_string();
            TagData {
                latest_tag_name: Some(latest_tag_name),
                commit_hash_str: Some(commit_hash_str),
                _phantom: std::marker::PhantomData,
            }
        } else {
            TagData {
                latest_tag_name: None,
                commit_hash_str: None,
                _phantom: std::marker::PhantomData,
            }
        };

    Ok(tag_data)
}

pub async fn run_cli(opt: RoochCli) -> RoochResult<String> {
    match opt.cmd {
        Command::Account(account) => account.execute().await,
        Command::Move(move_cli) => move_cli.execute().await,
        Command::Server(server) => server.execute().await,
        Command::Init(init) => init.execute_serialized().await,
        Command::State(state) => state.execute_serialized().await,
        Command::Object(object) => object.execute_serialized().await,
        Command::Resource(resource) => resource.execute_serialized().await,
        Command::Transaction(transation) => transation.execute().await,
        Command::Event(event) => event.execute().await,
        Command::ABI(abi) => abi.execute().await,
        Command::Env(env) => env.execute().await,
        Command::SessionKey(session_key) => session_key.execute().await,
        Command::Rpc(rpc) => rpc.execute().await,
        Command::Statedb(statedb) => statedb.execute().await,
        Command::Indexer(indexer) => indexer.execute().await,
        Command::Genesis(genesis) => genesis.execute().await,
    }
}
