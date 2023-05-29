//! Blockchain filter

use starcoin_vm_types::language_storage::type_tag_match;

use crate::account_address::AccountAddress;
use crate::block::BlockNumber;
use crate::contract_event::ContractEvent;
use crate::event::EventKey;
use crate::language_storage::TypeTag;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    /// Blockchain will be searched from this block.
    pub from_block: BlockNumber,
    /// Till this block.
    pub to_block: BlockNumber,

    /// Search events.
    ///
    /// If empty, match all.
    /// If specified, event must produced from one of the event keys.
    pub event_keys: Vec<EventKey>,

    /// Account addresses which event comes from.
    /// match if event belongs to any og the addresses.
    /// if `addrs` is empty, event always match.
    pub addrs: Vec<AccountAddress>,
    /// type tags of the event.
    /// match if the event is any type of the type tags.
    /// if `type_tags` is empty, event always match.
    pub type_tags: Vec<TypeTag>,

    /// Events limit
    ///
    /// If None, return all events
    /// If specified, should only return *last* `n` events.
    pub limit: Option<usize>,
    /// return events in reverse order.
    pub reverse: bool,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            from_block: 0,
            to_block: 0,
            event_keys: vec![],
            type_tags: vec![],
            addrs: vec![],
            limit: None,
            reverse: true,
        }
    }
}

impl Filter {
    pub fn matching(&self, block_number: BlockNumber, e: &ContractEvent) -> bool {
        if self.from_block <= block_number
            && block_number <= self.to_block
            && (self.event_keys.is_empty() || self.event_keys.contains(e.key()))
            && (self.addrs.is_empty() || self.addrs.contains(&e.key().get_creator_address()))
        {
            if self.type_tags.is_empty() {
                return true;
            } else {
                for filter_type_tag in &self.type_tags {
                    if type_tag_match(filter_type_tag, e.type_tag()) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, Hash)]
pub struct EventParams {
    #[serde(flatten)]
    pub filter: EventFilter,
    #[serde(default)]
    pub decode: bool,
}

/// Filter
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, Hash, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EventFilter {
    /// From Block
    #[serde(default)]
    pub from_block: Option<u64>,
    /// To Block
    #[serde(default)]
    pub to_block: Option<u64>,
    /// Event keys
    /// /// if `event_keys` is empty, event always match.
    #[serde(default)]
    pub event_keys: Option<Vec<EventKey>>,
    /// Account addresses which event comes from.
    /// match if event belongs to any og the addresses.
    /// if `addrs` is empty, event always match.
    #[serde(default)]
    pub addrs: Option<Vec<AccountAddress>>,
    /// type tags of the event.
    /// match if the event is any type of the type tags.
    /// /// if `type_tags` is empty, event always match.
    #[serde(default)]
    pub type_tags: Option<Vec<TypeTagView>>,
    /// Limit: from latest to oldest
    #[serde(default)]
    pub limit: Option<usize>,
}

impl TryInto<Filter> for EventFilter {
    type Error = JsonRpcError;

    fn try_into(self) -> std::result::Result<Filter, Self::Error> {
        match (self.from_block, self.to_block) {
            (Some(f), Some(t)) if f > t => {
                return Err(errors::invalid_params(
                    "fromBlock",
                    "fromBlock should not greater than toBlock",
                ));
            }
            _ => {}
        }
        Ok(Filter {
            from_block: self.from_block.unwrap_or(0),
            to_block: self.to_block.unwrap_or(std::u64::MAX),
            event_keys: self.event_keys.unwrap_or_default(),
            addrs: self.addrs.unwrap_or_default(),
            type_tags: self
                .type_tags
                .unwrap_or_default()
                .into_iter()
                .map(|t| t.0)
                .collect(),
            limit: self.limit,
            reverse: true,
        })
    }
}
