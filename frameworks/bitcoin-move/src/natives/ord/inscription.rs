// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
// Code from https://github.com/ordinals/ord/

use crate::natives::ord::envelope::Envelope;
use crate::natives::ord::tag::Tag;
use bitcoin::constants::MAX_SCRIPT_ELEMENT_SIZE;
use bitcoin::{hashes::Hash, Txid, Witness};
use moveos_types::move_std::string::MoveString;
use rooch_types::bitcoin::ord::InscriptionID;
use {
    super::envelope,
    super::inscription_id::InscriptionId,
    super::media::Media,
    axum::http::HeaderValue,
    bitcoin::{
        blockdata::{
            opcodes,
            script::{self},
        },
        ScriptBuf,
    },
    serde::{Deserialize, Serialize},
    std::str,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, Default)]
pub(crate) struct Inscription {
    pub body: Option<Vec<u8>>,
    pub content_encoding: Option<Vec<u8>>,
    pub content_type: Option<Vec<u8>>,
    pub duplicate_field: bool,
    pub incomplete_field: bool,
    pub metadata: Option<Vec<u8>>,
    pub metaprotocol: Option<Vec<u8>>,
    pub parents: Vec<Vec<u8>>,
    pub pointer: Option<Vec<u8>>,
    pub unrecognized_even_field: bool,
    pub rune: Option<Vec<u8>>,
}

impl From<Envelope<Inscription>>
    for rooch_types::bitcoin::ord::Envelope<rooch_types::bitcoin::ord::InscriptionRecord>
{
    fn from(val: Envelope<Inscription>) -> Self {
        Self {
            input: val.input,
            offset: val.offset,
            pushnum: val.pushnum,
            stutter: val.stutter,
            payload: val.payload.into(),
        }
    }
}

impl From<Inscription> for rooch_types::bitcoin::ord::InscriptionRecord {
    fn from(val: Inscription) -> Self {
        let content_encoding = val
            .content_encoding()
            .and_then(|v| v.to_str().ok().map(|s| MoveString::from(s.to_owned())))
            .into();
        let content_type = val
            .content_type()
            .map(|s| MoveString::from(s.to_owned()))
            .into();
        let metaprotocol = val
            .metaprotocol()
            .map(|s| MoveString::from(s.to_owned()))
            .into();
        let parents = val.parents().into_iter().map(InscriptionID::from).collect();
        let pointer = val.pointer().into();
        rooch_types::bitcoin::ord::InscriptionRecord {
            body: val.body.unwrap_or_default(),
            content_encoding,
            content_type,
            duplicate_field: val.duplicate_field,
            incomplete_field: val.incomplete_field,
            metadata: val.metadata.unwrap_or_default(),
            metaprotocol,
            parents,
            pointer,
            unrecognized_even_field: val.unrecognized_even_field,
            rune: None,
        }
    }
}

impl Inscription {
    #[cfg(test)]
    pub(crate) fn new(content_type: Option<Vec<u8>>, body: Option<Vec<u8>>) -> Self {
        Self {
            content_type,
            body,
            ..Default::default()
        }
    }

    fn pointer_value(pointer: u64) -> Vec<u8> {
        let mut bytes = pointer.to_le_bytes().to_vec();

        while bytes.last().copied() == Some(0) {
            bytes.pop();
        }

        bytes
    }

    pub(crate) fn append_reveal_script_to_builder(
        &self,
        mut builder: script::Builder,
    ) -> script::Builder {
        builder = builder
            .push_opcode(opcodes::OP_FALSE)
            .push_opcode(opcodes::all::OP_IF)
            .push_slice(envelope::PROTOCOL_ID);

        Tag::ContentType.append(&mut builder, &self.content_type);
        Tag::ContentEncoding.append(&mut builder, &self.content_encoding);
        Tag::Metaprotocol.append(&mut builder, &self.metaprotocol);
        Tag::Parent.append_array(&mut builder, &self.parents);
        Tag::Pointer.append(&mut builder, &self.pointer);
        Tag::Metadata.append(&mut builder, &self.metadata);

        if let Some(body) = &self.body {
            builder = builder.push_slice(envelope::BODY_TAG);
            for chunk in body.chunks(MAX_SCRIPT_ELEMENT_SIZE) {
                builder = builder.push_slice::<&script::PushBytes>(chunk.try_into().unwrap());
            }
        }

        builder.push_opcode(opcodes::all::OP_ENDIF)
    }

    pub(crate) fn append_reveal_script(&self, builder: script::Builder) -> ScriptBuf {
        self.append_reveal_script_to_builder(builder).into_script()
    }

    pub(crate) fn append_batch_reveal_script_to_builder(
        inscriptions: &[Inscription],
        mut builder: script::Builder,
    ) -> script::Builder {
        for inscription in inscriptions {
            builder = inscription.append_reveal_script_to_builder(builder);
        }

        builder
    }

    pub(crate) fn append_batch_reveal_script(
        inscriptions: &[Inscription],
        builder: script::Builder,
    ) -> ScriptBuf {
        Inscription::append_batch_reveal_script_to_builder(inscriptions, builder).into_script()
    }

    pub(crate) fn media(&self) -> Media {
        if self.body.is_none() {
            return Media::Unknown;
        }

        let Some(content_type) = self.content_type() else {
            return Media::Unknown;
        };

        content_type.parse().unwrap_or(Media::Unknown)
    }

    pub(crate) fn body(&self) -> Option<&[u8]> {
        Some(self.body.as_ref()?)
    }

    pub(crate) fn into_body(self) -> Option<Vec<u8>> {
        self.body
    }

    pub(crate) fn content_length(&self) -> Option<usize> {
        Some(self.body()?.len())
    }

    pub(crate) fn content_type(&self) -> Option<&str> {
        str::from_utf8(self.content_type.as_ref()?).ok()
    }

    pub(crate) fn content_encoding(&self) -> Option<HeaderValue> {
        HeaderValue::from_str(str::from_utf8(self.content_encoding.as_ref()?).unwrap_or_default())
            .ok()
    }

    pub(crate) fn metadata(&self) -> Option<&Vec<u8>> {
        self.metadata.as_ref()
    }

    pub(crate) fn metaprotocol(&self) -> Option<&str> {
        str::from_utf8(self.metaprotocol.as_ref()?).ok()
    }

    pub(crate) fn parents(&self) -> Vec<InscriptionId> {
        self.parents
            .iter()
            .filter_map(|parent| Self::inscription_id_field(Some(parent)))
            .collect()
    }

    fn inscription_id_field(field: Option<&[u8]>) -> Option<InscriptionId> {
        let value = field.as_ref()?;

        if value.len() < Txid::LEN {
            return None;
        }

        if value.len() > Txid::LEN + 4 {
            return None;
        }

        let (txid, index) = value.split_at(Txid::LEN);

        if let Some(last) = index.last() {
            // Accept fixed length encoding with 4 bytes (with potential trailing zeroes)
            // or variable length (no trailing zeroes)
            if index.len() != 4 && *last == 0 {
                return None;
            }
        }

        let txid = Txid::from_slice(txid).unwrap();

        let index = [
            index.first().copied().unwrap_or(0),
            index.get(1).copied().unwrap_or(0),
            index.get(2).copied().unwrap_or(0),
            index.get(3).copied().unwrap_or(0),
        ];

        let index = u32::from_le_bytes(index);

        Some(InscriptionId { txid, index })
    }

    pub(crate) fn pointer(&self) -> Option<u64> {
        let value = self.pointer.as_ref()?;

        if value.iter().skip(8).copied().any(|byte| byte != 0) {
            return None;
        }

        let pointer = [
            value.first().copied().unwrap_or(0),
            value.get(1).copied().unwrap_or(0),
            value.get(2).copied().unwrap_or(0),
            value.get(3).copied().unwrap_or(0),
            value.get(4).copied().unwrap_or(0),
            value.get(5).copied().unwrap_or(0),
            value.get(6).copied().unwrap_or(0),
            value.get(7).copied().unwrap_or(0),
        ];

        Some(u64::from_le_bytes(pointer))
    }

    pub(crate) fn to_witness(&self) -> Witness {
        let builder = script::Builder::new();

        let script = self.append_reveal_script(builder);

        let mut witness = Witness::new();

        witness.push(script);
        witness.push([]);

        witness
    }

    pub(crate) fn hidden(&self) -> bool {
        let Some(content_type) = self.content_type() else {
            return false;
        };

        if content_type != "text/plain" && content_type != "text/plain;charset=utf-8" {
            return false;
        }

        let Some(body) = &self.body else {
            return false;
        };

        let Ok(text) = str::from_utf8(body) else {
            return false;
        };

        let trimmed = text.trim();

        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return true;
        }

        if trimmed.starts_with("gib bc1") {
            return true;
        }

        if trimmed.ends_with(".bitmap") {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use {super::super::test::*, super::*};

    #[test]
    fn reveal_script_chunks_body() {
        assert_eq!(
            inscription("foo", [])
                .append_reveal_script(script::Builder::new())
                .instructions()
                .count(),
            7
        );

        assert_eq!(
            inscription("foo", [0; 1])
                .append_reveal_script(script::Builder::new())
                .instructions()
                .count(),
            8
        );

        assert_eq!(
            inscription("foo", [0; 520])
                .append_reveal_script(script::Builder::new())
                .instructions()
                .count(),
            8
        );

        assert_eq!(
            inscription("foo", [0; 521])
                .append_reveal_script(script::Builder::new())
                .instructions()
                .count(),
            9
        );

        assert_eq!(
            inscription("foo", [0; 1040])
                .append_reveal_script(script::Builder::new())
                .instructions()
                .count(),
            9
        );

        assert_eq!(
            inscription("foo", [0; 1041])
                .append_reveal_script(script::Builder::new())
                .instructions()
                .count(),
            10
        );
    }

    #[test]
    fn reveal_script_chunks_metadata() {
        assert_eq!(
            Inscription {
                metadata: None,
                ..Default::default()
            }
            .append_reveal_script(script::Builder::new())
            .instructions()
            .count(),
            4
        );

        assert_eq!(
            Inscription {
                metadata: Some(Vec::new()),
                ..Default::default()
            }
            .append_reveal_script(script::Builder::new())
            .instructions()
            .count(),
            4
        );

        assert_eq!(
            Inscription {
                metadata: Some(vec![0; 1]),
                ..Default::default()
            }
            .append_reveal_script(script::Builder::new())
            .instructions()
            .count(),
            6
        );

        assert_eq!(
            Inscription {
                metadata: Some(vec![0; 520]),
                ..Default::default()
            }
            .append_reveal_script(script::Builder::new())
            .instructions()
            .count(),
            6
        );

        assert_eq!(
            Inscription {
                metadata: Some(vec![0; 521]),
                ..Default::default()
            }
            .append_reveal_script(script::Builder::new())
            .instructions()
            .count(),
            8
        );
    }

    #[test]
    fn inscription_with_no_parent_field_has_no_parent() {
        assert!(Inscription {
            parents: vec![],
            ..Default::default()
        }
        .parents()
        .is_empty());
    }

    #[test]
    fn inscription_with_parent_field_shorter_than_txid_length_has_no_parents() {
        assert!(Inscription {
            parents: vec![],
            ..Default::default()
        }
        .parents()
        .is_empty());
    }

    #[test]
    fn inscription_with_parent_field_longer_than_txid_and_index_has_no_parents() {
        assert!(Inscription {
            parents: vec![vec![1; 37]],
            ..Default::default()
        }
        .parents()
        .is_empty());
    }

    #[test]
    fn inscription_with_parent_field_index_with_trailing_zeroes_and_fixed_length_has_parents() {
        let mut parent = vec![1; 36];

        parent[35] = 0;

        assert!(
            !(Inscription {
                parents: vec![parent],
                ..Default::default()
            }
            .parents()
            .is_empty())
        );
    }

    #[test]
    fn inscription_with_parent_field_index_with_trailing_zeroes_and_variable_length_has_no_parents()
    {
        let mut parent = vec![1; 35];

        parent[34] = 0;

        assert!(Inscription {
            parents: vec![parent],
            ..Default::default()
        }
        .parents()
        .is_empty());
    }

    #[test]
    fn inscription_parent_txid_is_deserialized_correctly() {
        assert_eq!(
            Inscription {
                parents: vec![vec![
                    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
                    0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19,
                    0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
                ]],
                ..Default::default()
            }
            .parents(),
            [
                "1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100i0"
                    .parse()
                    .unwrap()
            ],
        );
    }

    #[test]
    fn inscription_parent_with_zero_byte_index_field_is_deserialized_correctly() {
        assert_eq!(
            Inscription {
                parents: vec![vec![1; 32]],
                ..Default::default()
            }
            .parents(),
            [
                "0101010101010101010101010101010101010101010101010101010101010101i0"
                    .parse()
                    .unwrap()
            ],
        );
    }

    #[test]
    fn inscription_parent_with_one_byte_index_field_is_deserialized_correctly() {
        assert_eq!(
            Inscription {
                parents: vec![vec![
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01
                ]],
                ..Default::default()
            }
            .parents(),
            [
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffi1"
                    .parse()
                    .unwrap()
            ],
        );
    }

    #[test]
    fn inscription_parent_with_two_byte_index_field_is_deserialized_correctly() {
        assert_eq!(
            Inscription {
                parents: vec![vec![
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01, 0x02
                ]],
                ..Default::default()
            }
            .parents(),
            [
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffi513"
                    .parse()
                    .unwrap()
            ],
        );
    }

    #[test]
    fn inscription_parent_with_three_byte_index_field_is_deserialized_correctly() {
        assert_eq!(
            Inscription {
                parents: vec![vec![
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01, 0x02, 0x03
                ]],
                ..Default::default()
            }
            .parents(),
            [
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffi197121"
                    .parse()
                    .unwrap()
            ],
        );
    }

    #[test]
    fn inscription_parent_with_four_byte_index_field_is_deserialized_correctly() {
        assert_eq!(
            Inscription {
                parents: vec![vec![
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01, 0x02, 0x03, 0x04,
                ]],
                ..Default::default()
            }
            .parents(),
            [
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffi67305985"
                    .parse()
                    .unwrap()
            ],
        );
    }

    #[test]
    fn metadata_function_decodes_metadata() {
        assert_eq!(
            Inscription {
                metadata: Some(vec![0x44, 0, 1, 2, 3]),
                ..Default::default()
            }
            .metadata()
            .unwrap(),
            &vec![0x44, 0, 1, 2, 3],
        );
    }

    #[test]
    fn metadata_function_returns_none_if_no_metadata() {
        assert_eq!(
            Inscription {
                metadata: None,
                ..Default::default()
            }
            .metadata(),
            None,
        );
    }

    #[test]
    fn metadata_function_returns_none_if_metadata_fails_to_parse() {
        assert_eq!(
            Inscription {
                metadata: Some(vec![0x44]),
                ..Default::default()
            }
            .metadata()
            .unwrap(),
            &vec![0x44],
        );
    }

    #[test]
    fn pointer_decode() {
        assert_eq!(
            Inscription {
                pointer: None,
                ..Default::default()
            }
            .pointer(),
            None
        );
        assert_eq!(
            Inscription {
                pointer: Some(vec![0]),
                ..Default::default()
            }
            .pointer(),
            Some(0),
        );
        assert_eq!(
            Inscription {
                pointer: Some(vec![1, 2, 3, 4, 5, 6, 7, 8]),
                ..Default::default()
            }
            .pointer(),
            Some(0x0807060504030201),
        );
        assert_eq!(
            Inscription {
                pointer: Some(vec![1, 2, 3, 4, 5, 6]),
                ..Default::default()
            }
            .pointer(),
            Some(0x0000060504030201),
        );
        assert_eq!(
            Inscription {
                pointer: Some(vec![1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0]),
                ..Default::default()
            }
            .pointer(),
            Some(0x0807060504030201),
        );
        assert_eq!(
            Inscription {
                pointer: Some(vec![1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 1]),
                ..Default::default()
            }
            .pointer(),
            None,
        );
        assert_eq!(
            Inscription {
                pointer: Some(vec![1, 2, 3, 4, 5, 6, 7, 8, 1]),
                ..Default::default()
            }
            .pointer(),
            None,
        );
    }

    #[test]
    fn pointer_encode() {
        assert_eq!(
            Inscription {
                pointer: None,
                ..Default::default()
            }
            .to_witness(),
            envelope(&[b"ord"]),
        );

        assert_eq!(
            Inscription {
                pointer: Some(vec![1, 2, 3]),
                ..Default::default()
            }
            .to_witness(),
            envelope(&[b"ord", &[2], &[1, 2, 3]]),
        );
    }

    #[test]
    fn hidden() {
        #[track_caller]
        fn case(content_type: Option<&str>, body: Option<&str>, expected: bool) {
            assert_eq!(
                Inscription {
                    content_type: content_type.map(|content_type| content_type.as_bytes().into()),
                    body: body.map(|content_type| content_type.as_bytes().into()),
                    ..Default::default()
                }
                .hidden(),
                expected
            );
        }

        case(None, None, false);
        case(Some("foo"), None, false);
        case(Some("foo"), Some("{}"), false);
        case(Some("text/plain"), None, false);
        case(Some("text/plain"), Some("foo{}bar"), false);

        case(Some("text/plain"), Some("foo.bitmap"), true);
        case(Some("text/plain"), Some("gib bc1"), true);
        case(Some("text/plain"), Some("{}"), true);
        case(Some("text/plain"), Some(" {} "), true);
        case(Some("text/plain;charset=utf-8"), Some("foo.bitmap"), true);

        assert!(!Inscription {
            content_type: Some("text/plain".as_bytes().into()),
            body: Some(b"{\xc3\x28}".as_slice().into()),
            ..Default::default()
        }
        .hidden());
    }
}
