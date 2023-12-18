// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{inscription::Inscription, inscription_id::InscriptionId};
use bitcoin::{
    address::NetworkUnchecked,
    opcodes,
    script::{self, PushBytesBuf},
    Address, Amount, BlockHash, OutPoint, ScriptBuf, Sequence, TxIn, TxOut, Txid, Witness,
};

#[macro_export]
macro_rules! assert_regex_match {
    ($value:expr, $pattern:expr $(,)?) => {
        let regex = Regex::new(&format!("^(?s){}$", $pattern)).unwrap();
        let string = $value.to_string();

        if !regex.is_match(string.as_ref()) {
            panic!(
                "Regex:\n\n{}\n\n…did not match string:\n\n{}",
                regex, string
            );
        }
    };
}

#[macro_export]
macro_rules! assert_matches {
  ($expression:expr, $( $pattern:pat_param )|+ $( if $guard:expr )? $(,)?) => {
    match $expression {
      $( $pattern )|+ $( if $guard )? => {}
      left => panic!(
        "assertion failed: (left ~= right)\n  left: `{:?}`\n right: `{}`",
        left,
        stringify!($($pattern)|+ $(if $guard)?)
      ),
    }
  }
}

pub(crate) fn blockhash(n: u64) -> BlockHash {
    let hex = format!("{n:x}");

    if hex.is_empty() || hex.len() > 1 {
        panic!();
    }

    hex.repeat(64).parse().unwrap()
}

pub(crate) fn txid(n: u64) -> Txid {
    let hex = format!("{n:x}");

    if hex.is_empty() || hex.len() > 1 {
        panic!();
    }

    hex.repeat(64).parse().unwrap()
}

pub(crate) fn outpoint(n: u64) -> OutPoint {
    format!("{}:{}", txid(n), n).parse().unwrap()
}

// pub(crate) fn satpoint(n: u64, offset: u64) -> SatPoint {
//   SatPoint {
//     outpoint: outpoint(n),
//     offset,
//   }
// }

pub(crate) fn address() -> Address {
    "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        .parse::<Address<NetworkUnchecked>>()
        .unwrap()
        .assume_checked()
}

pub(crate) fn recipient() -> Address {
    "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
        .parse::<Address<NetworkUnchecked>>()
        .unwrap()
        .assume_checked()
}

pub(crate) fn change(n: u64) -> Address {
    match n {
        0 => "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww",
        1 => "tb1qakxxzv9n7706kc3xdcycrtfv8cqv62hnwexc0l",
        2 => "tb1qxz9yk0td0yye009gt6ayn7jthz5p07a75luryg",
        3 => "tb1qe62s57n77pfhlw2vtqlhm87dwj75l6fguavjjq",
        _ => panic!(),
    }
    .parse::<Address<NetworkUnchecked>>()
    .unwrap()
    .assume_checked()
}

pub(crate) fn tx_in(previous_output: OutPoint) -> TxIn {
    TxIn {
        previous_output,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
    }
}

pub(crate) fn tx_out(value: u64, address: Address) -> TxOut {
    TxOut {
        value: Amount::from_sat(value),
        script_pubkey: address.script_pubkey(),
    }
}

pub(crate) struct InscriptionTemplate {
    pub(crate) parent: Option<InscriptionId>,
}

impl From<InscriptionTemplate> for Inscription {
    fn from(template: InscriptionTemplate) -> Self {
        Self {
            parent: template.parent.map(|id| id.parent_value()),
            ..Default::default()
        }
    }
}

pub(crate) fn inscription(content_type: &str, body: impl AsRef<[u8]>) -> Inscription {
    Inscription::new(Some(content_type.into()), Some(body.as_ref().into()))
}

pub(crate) fn inscription_id(n: u32) -> InscriptionId {
    let hex = format!("{n:x}");

    if hex.is_empty() || hex.len() > 1 {
        panic!();
    }

    format!("{}i{n}", hex.repeat(64)).parse().unwrap()
}

pub(crate) fn envelope(payload: &[&[u8]]) -> Witness {
    let mut builder = script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF);

    for data in payload {
        let mut buf = PushBytesBuf::new();
        buf.extend_from_slice(data).unwrap();
        builder = builder.push_slice(buf);
    }

    let script = builder.push_opcode(opcodes::all::OP_ENDIF).into_script();

    Witness::from_slice(&[script.into_bytes(), Vec::new()])
}
