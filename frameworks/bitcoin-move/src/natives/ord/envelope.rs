// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
// Code from https://github.com/ordinals/ord/

use bitcoin::script::Instruction::{Op, PushBytes};
use bitcoin::{Script, Transaction};
use std::collections::BTreeMap;
use std::iter::Peekable;
use {
    super::inscription::Inscription,
    bitcoin::blockdata::{
        opcodes,
        script::{self, Instruction, Instructions},
    },
};

pub(crate) const PROTOCOL_ID: [u8; 3] = *b"ord";

pub(crate) const BODY_TAG: [u8; 0] = [];
pub(crate) const CONTENT_TYPE_TAG: [u8; 1] = [1];
pub(crate) const POINTER_TAG: [u8; 1] = [2];
pub(crate) const PARENT_TAG: [u8; 1] = [3];
pub(crate) const METADATA_TAG: [u8; 1] = [5];
pub(crate) const METAPROTOCOL_TAG: [u8; 1] = [7];
pub(crate) const CONTENT_ENCODING_TAG: [u8; 1] = [9];
pub(crate) const RUNE_TAG: [u8; 1] = [13];

type Result<T> = std::result::Result<T, script::Error>;
pub type RawEnvelope = Envelope<Vec<Vec<u8>>>;
pub type ParsedEnvelope = Envelope<Inscription>;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Envelope<T> {
    pub payload: T,
    pub input: u32,
    pub offset: u32,
    pub pushnum: bool,
    pub stutter: bool,
}

fn remove_field(fields: &mut BTreeMap<&[u8], Vec<&[u8]>>, field: &[u8]) -> Option<Vec<u8>> {
    let values = fields.get_mut(field)?;

    if values.is_empty() {
        None
    } else {
        let value = values.remove(0).to_vec();

        if values.is_empty() {
            fields.remove(field);
        }

        Some(value)
    }
}

fn remove_array_field(fields: &mut BTreeMap<&[u8], Vec<&[u8]>>, field: &[u8]) -> Vec<Vec<u8>> {
    fields
        .remove(field)
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.to_vec())
        .collect()
}

fn remove_and_concatenate_field(
    fields: &mut BTreeMap<&[u8], Vec<&[u8]>>,
    field: &[u8],
) -> Option<Vec<u8>> {
    let value = fields.remove(field)?;

    if value.is_empty() {
        None
    } else {
        Some(value.into_iter().flatten().cloned().collect())
    }
}

impl From<RawEnvelope> for ParsedEnvelope {
    fn from(envelope: RawEnvelope) -> Self {
        let body = envelope
            .payload
            .iter()
            .enumerate()
            .position(|(i, push)| i % 2 == 0 && push.is_empty());

        let mut fields: BTreeMap<&[u8], Vec<&[u8]>> = BTreeMap::new();

        let mut incomplete_field = false;

        for item in envelope.payload[..body.unwrap_or(envelope.payload.len())].chunks(2) {
            match item {
                [key, value] => fields.entry(key).or_default().push(value),
                _ => incomplete_field = true,
            }
        }

        let duplicate_field = fields.iter().any(|(_key, values)| values.len() > 1);

        let content_encoding = remove_field(&mut fields, &CONTENT_ENCODING_TAG);
        let content_type = remove_field(&mut fields, &CONTENT_TYPE_TAG);
        let metadata = remove_and_concatenate_field(&mut fields, &METADATA_TAG);
        let metaprotocol = remove_field(&mut fields, &METAPROTOCOL_TAG);
        let parents = remove_array_field(&mut fields, &PARENT_TAG);
        let pointer = remove_field(&mut fields, &POINTER_TAG);
        let rune = remove_field(&mut fields, &RUNE_TAG);

        let unrecognized_even_field = fields
            .keys()
            .any(|tag| tag.first().map(|lsb| lsb % 2 == 0).unwrap_or_default());

        Self {
            payload: Inscription {
                body: body.map(|i| {
                    envelope.payload[i + 1..]
                        .iter()
                        .flatten()
                        .cloned()
                        .collect()
                }),
                content_encoding,
                content_type,
                duplicate_field,
                incomplete_field,
                metadata,
                metaprotocol,
                parents,
                pointer,
                unrecognized_even_field,
                rune,
            },
            input: envelope.input,
            offset: envelope.offset,
            pushnum: envelope.pushnum,
            stutter: envelope.stutter,
        }
    }
}

impl ParsedEnvelope {
    pub fn from_transaction(transaction: &Transaction) -> Vec<Self> {
        RawEnvelope::from_transaction(transaction)
            .into_iter()
            .map(|envelope| envelope.into())
            .collect()
    }
}

impl RawEnvelope {
    pub(crate) fn from_transaction(transaction: &Transaction) -> Vec<Self> {
        let mut envelopes = Vec::new();

        for (i, input) in transaction.input.iter().enumerate() {
            if let Some(tapscript) = input.witness.tapscript() {
                if let Ok(input_envelopes) = Self::from_tapscript(tapscript, i) {
                    envelopes.extend(input_envelopes);
                }
            }
        }

        envelopes
    }

    pub(crate) fn from_tapscript(tapscript: &Script, input: usize) -> Result<Vec<Self>> {
        let mut envelopes = Vec::new();

        let mut instructions = tapscript.instructions().peekable();

        let mut stuttered = false;
        while let Some(instruction) = instructions.next().transpose()? {
            if instruction == Instruction::PushBytes((&[]).into()) {
                let (stutter, envelope) =
                    Self::from_instructions(&mut instructions, input, envelopes.len(), stuttered)?;
                if let Some(envelope) = envelope {
                    envelopes.push(envelope);
                } else {
                    stuttered = stutter;
                }
            }
        }

        Ok(envelopes)
    }

    fn accept(instructions: &mut Peekable<Instructions>, instruction: Instruction) -> Result<bool> {
        if instructions.peek() == Some(&Ok(instruction)) {
            instructions.next().transpose()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn from_instructions(
        instructions: &mut Peekable<Instructions>,
        input: usize,
        offset: usize,
        stutter: bool,
    ) -> Result<(bool, Option<Self>)> {
        if !Self::accept(instructions, Op(opcodes::all::OP_IF))? {
            let stutter = instructions.peek() == Some(&Ok(PushBytes((&[]).into())));
            return Ok((stutter, None));
        }

        if !Self::accept(instructions, PushBytes((&PROTOCOL_ID).into()))? {
            let stutter = instructions.peek() == Some(&Ok(PushBytes((&[]).into())));
            return Ok((stutter, None));
        }

        let mut pushnum = false;

        let mut payload = Vec::new();

        loop {
            match instructions.next().transpose()? {
                None => return Ok((false, None)),
                Some(Op(opcodes::all::OP_ENDIF)) => {
                    return Ok((
                        false,
                        Some(Envelope {
                            input: input.try_into().unwrap(),
                            offset: offset.try_into().unwrap(),
                            payload,
                            pushnum,
                            stutter,
                        }),
                    ));
                }
                Some(Op(opcodes::all::OP_PUSHNUM_NEG1)) => {
                    pushnum = true;
                    payload.push(vec![0x81]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_1)) => {
                    pushnum = true;
                    payload.push(vec![1]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_2)) => {
                    pushnum = true;
                    payload.push(vec![2]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_3)) => {
                    pushnum = true;
                    payload.push(vec![3]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_4)) => {
                    pushnum = true;
                    payload.push(vec![4]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_5)) => {
                    pushnum = true;
                    payload.push(vec![5]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_6)) => {
                    pushnum = true;
                    payload.push(vec![6]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_7)) => {
                    pushnum = true;
                    payload.push(vec![7]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_8)) => {
                    pushnum = true;
                    payload.push(vec![8]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_9)) => {
                    pushnum = true;
                    payload.push(vec![9]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_10)) => {
                    pushnum = true;
                    payload.push(vec![10]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_11)) => {
                    pushnum = true;
                    payload.push(vec![11]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_12)) => {
                    pushnum = true;
                    payload.push(vec![12]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_13)) => {
                    pushnum = true;
                    payload.push(vec![13]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_14)) => {
                    pushnum = true;
                    payload.push(vec![14]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_15)) => {
                    pushnum = true;
                    payload.push(vec![15]);
                }
                Some(Op(opcodes::all::OP_PUSHNUM_16)) => {
                    pushnum = true;
                    payload.push(vec![16]);
                }
                Some(PushBytes(push)) => {
                    payload.push(push.as_bytes().to_vec());
                }
                Some(_) => return Ok((false, None)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::{transaction::Version, OutPoint, ScriptBuf, Sequence, TxIn, Witness};
    use {super::super::test::*, super::*, bitcoin::absolute::LockTime};

    fn parse(witnesses: &[Witness]) -> Vec<ParsedEnvelope> {
        ParsedEnvelope::from_transaction(&Transaction {
            version: Version::ONE,
            lock_time: LockTime::ZERO,
            input: witnesses
                .iter()
                .map(|witness| TxIn {
                    previous_output: OutPoint::null(),
                    script_sig: ScriptBuf::new(),
                    sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                    witness: witness.clone(),
                })
                .collect(),
            output: Vec::new(),
        })
    }

    #[test]
    fn empty() {
        assert_eq!(parse(&[Witness::new()]), Vec::new())
    }

    #[test]
    fn ignore_key_path_spends() {
        assert_eq!(
            parse(&[Witness::from_slice(&[bitcoin::script::Builder::new()
                .push_opcode(bitcoin::opcodes::OP_FALSE)
                .push_opcode(bitcoin::opcodes::all::OP_IF)
                .push_slice(b"ord")
                .push_opcode(bitcoin::opcodes::all::OP_ENDIF)
                .into_script()
                .into_bytes()])]),
            Vec::new()
        );
    }

    #[test]
    fn ignore_key_path_spends_with_annex() {
        assert_eq!(
            parse(&[Witness::from_slice(&[
                bitcoin::script::Builder::new()
                    .push_opcode(bitcoin::opcodes::OP_FALSE)
                    .push_opcode(bitcoin::opcodes::all::OP_IF)
                    .push_slice(b"ord")
                    .push_opcode(bitcoin::opcodes::all::OP_ENDIF)
                    .into_script()
                    .into_bytes(),
                vec![0x50]
            ])]),
            Vec::new()
        );
    }

    #[test]
    fn parse_from_tapscript() {
        assert_eq!(
            parse(&[Witness::from_slice(&[
                bitcoin::script::Builder::new()
                    .push_opcode(bitcoin::opcodes::OP_FALSE)
                    .push_opcode(bitcoin::opcodes::all::OP_IF)
                    .push_slice(b"ord")
                    .push_opcode(bitcoin::opcodes::all::OP_ENDIF)
                    .into_script()
                    .into_bytes(),
                Vec::new()
            ])]),
            vec![ParsedEnvelope {
                ..Default::default()
            }]
        );
    }

    #[test]
    fn ignore_unparsable_scripts() {
        let mut script_bytes = bitcoin::script::Builder::new()
            .push_opcode(bitcoin::opcodes::OP_FALSE)
            .push_opcode(bitcoin::opcodes::all::OP_IF)
            .push_slice(b"ord")
            .push_opcode(bitcoin::opcodes::all::OP_ENDIF)
            .into_script()
            .into_bytes();
        script_bytes.push(0x01);

        assert_eq!(
            parse(&[Witness::from_slice(&[script_bytes, Vec::new()])]),
            Vec::new()
        );
    }

    #[test]
    fn no_inscription() {
        assert_eq!(
            parse(&[Witness::from_slice(&[
                ScriptBuf::new().into_bytes(),
                Vec::new()
            ])]),
            Vec::new()
        );
    }

    #[test]
    fn duplicate_field() {
        assert_eq!(
            parse(&[envelope(&[b"ord", &[255], &[], &[255], &[]])]),
            vec![ParsedEnvelope {
                payload: Inscription {
                    duplicate_field: true,
                    ..Default::default()
                },
                ..Default::default()
            }]
        );
    }

    #[test]
    fn with_content_type() {
        assert_eq!(
            parse(&[envelope(&[
                b"ord",
                &[1],
                b"text/plain;charset=utf-8",
                &[],
                b"ord",
            ])]),
            vec![ParsedEnvelope {
                payload: inscription("text/plain;charset=utf-8", "ord"),
                ..Default::default()
            }]
        );
    }

    #[test]
    fn with_content_encoding() {
        assert_eq!(
            parse(&[envelope(&[
                b"ord",
                &[1],
                b"text/plain;charset=utf-8",
                &[9],
                b"br",
                &[],
                b"ord",
            ])]),
            vec![ParsedEnvelope {
                payload: Inscription {
                    content_encoding: Some("br".as_bytes().to_vec()),
                    ..inscription("text/plain;charset=utf-8", "ord")
                },
                ..Default::default()
            }]
        );
    }

    #[test]
    fn with_unknown_tag() {
        assert_eq!(
            parse(&[envelope(&[
                b"ord",
                &[1],
                b"text/plain;charset=utf-8",
                &[11],
                b"bar",
                &[],
                b"ord",
            ])]),
            vec![ParsedEnvelope {
                payload: inscription("text/plain;charset=utf-8", "ord"),
                ..Default::default()
            }]
        );
    }

    #[test]
    fn no_body() {
        assert_eq!(
            parse(&[envelope(&[b"ord", &[1], b"text/plain;charset=utf-8"])]),
            vec![ParsedEnvelope {
                payload: Inscription {
                    content_type: Some(b"text/plain;charset=utf-8".to_vec()),
                    ..Default::default()
                },
                ..Default::default()
            }],
        );
    }

    #[test]
    fn no_content_type() {
        assert_eq!(
            parse(&[envelope(&[b"ord", &[], b"foo"])]),
            vec![ParsedEnvelope {
                payload: Inscription {
                    body: Some(b"foo".to_vec()),
                    ..Default::default()
                },
                ..Default::default()
            }],
        );
    }

    #[test]
    fn valid_body_in_multiple_pushes() {
        assert_eq!(
            parse(&[envelope(&[
                b"ord",
                &[1],
                b"text/plain;charset=utf-8",
                &[],
                b"foo",
                b"bar"
            ])]),
            vec![ParsedEnvelope {
                payload: inscription("text/plain;charset=utf-8", "foobar"),
                ..Default::default()
            }],
        );
    }

    #[test]
    fn valid_body_in_zero_pushes() {
        assert_eq!(
            parse(&[envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[]])]),
            vec![ParsedEnvelope {
                payload: inscription("text/plain;charset=utf-8", ""),
                ..Default::default()
            }]
        );
    }

    #[test]
    fn valid_body_in_multiple_empty_pushes() {
        assert_eq!(
            parse(&[envelope(&[
                b"ord",
                &[1],
                b"text/plain;charset=utf-8",
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
            ])]),
            vec![ParsedEnvelope {
                payload: inscription("text/plain;charset=utf-8", ""),
                ..Default::default()
            }],
        );
    }

    #[test]
    fn valid_ignore_trailing() {
        let script = script::Builder::new()
            .push_opcode(opcodes::OP_FALSE)
            .push_opcode(opcodes::all::OP_IF)
            .push_slice(b"ord")
            .push_slice([1])
            .push_slice(b"text/plain;charset=utf-8")
            .push_slice([])
            .push_slice(b"ord")
            .push_opcode(opcodes::all::OP_ENDIF)
            .push_opcode(opcodes::all::OP_CHECKSIG)
            .into_script();

        assert_eq!(
            parse(&[Witness::from_slice(&[script.into_bytes(), Vec::new()])]),
            vec![ParsedEnvelope {
                payload: inscription("text/plain;charset=utf-8", "ord"),
                ..Default::default()
            }],
        );
    }

    #[test]
    fn valid_ignore_preceding() {
        let script = script::Builder::new()
            .push_opcode(opcodes::all::OP_CHECKSIG)
            .push_opcode(opcodes::OP_FALSE)
            .push_opcode(opcodes::all::OP_IF)
            .push_slice(b"ord")
            .push_slice([1])
            .push_slice(b"text/plain;charset=utf-8")
            .push_slice([])
            .push_slice(b"ord")
            .push_opcode(opcodes::all::OP_ENDIF)
            .into_script();

        assert_eq!(
            parse(&[Witness::from_slice(&[script.into_bytes(), Vec::new()])]),
            vec![ParsedEnvelope {
                payload: inscription("text/plain;charset=utf-8", "ord"),
                ..Default::default()
            }],
        );
    }

    #[test]
    fn multiple_inscriptions_in_a_single_witness() {
        let script = script::Builder::new()
            .push_opcode(opcodes::OP_FALSE)
            .push_opcode(opcodes::all::OP_IF)
            .push_slice(b"ord")
            .push_slice([1])
            .push_slice(b"text/plain;charset=utf-8")
            .push_slice([])
            .push_slice(b"foo")
            .push_opcode(opcodes::all::OP_ENDIF)
            .push_opcode(opcodes::OP_FALSE)
            .push_opcode(opcodes::all::OP_IF)
            .push_slice(b"ord")
            .push_slice([1])
            .push_slice(b"text/plain;charset=utf-8")
            .push_slice([])
            .push_slice(b"bar")
            .push_opcode(opcodes::all::OP_ENDIF)
            .into_script();

        assert_eq!(
            parse(&[Witness::from_slice(&[script.into_bytes(), Vec::new()])]),
            vec![
                ParsedEnvelope {
                    payload: inscription("text/plain;charset=utf-8", "foo"),
                    ..Default::default()
                },
                ParsedEnvelope {
                    payload: inscription("text/plain;charset=utf-8", "bar"),
                    offset: 1,
                    ..Default::default()
                },
            ],
        );
    }

    #[test]
    fn invalid_utf8_does_not_render_inscription_invalid() {
        assert_eq!(
            parse(&[envelope(&[
                b"ord",
                &[1],
                b"text/plain;charset=utf-8",
                &[],
                &[0b10000000]
            ])]),
            vec![ParsedEnvelope {
                payload: inscription("text/plain;charset=utf-8", [0b10000000]),
                ..Default::default()
            },],
        );
    }

    #[test]
    fn no_endif() {
        let script = script::Builder::new()
            .push_opcode(opcodes::OP_FALSE)
            .push_opcode(opcodes::all::OP_IF)
            .push_slice(b"ord")
            .into_script();

        assert_eq!(
            parse(&[Witness::from_slice(&[script.into_bytes(), Vec::new()])]),
            Vec::new(),
        );
    }

    #[test]
    fn no_op_false() {
        let script = script::Builder::new()
            .push_opcode(opcodes::all::OP_IF)
            .push_slice(b"ord")
            .push_opcode(opcodes::all::OP_ENDIF)
            .into_script();

        assert_eq!(
            parse(&[Witness::from_slice(&[script.into_bytes(), Vec::new()])]),
            Vec::new(),
        );
    }

    #[test]
    fn empty_envelope() {
        assert_eq!(parse(&[envelope(&[])]), Vec::new());
    }

    #[test]
    fn wrong_protocol_identifier() {
        assert_eq!(parse(&[envelope(&[b"foo"])]), Vec::new());
    }

    #[test]
    fn extract_from_transaction() {
        assert_eq!(
            parse(&[envelope(&[
                b"ord",
                &[1],
                b"text/plain;charset=utf-8",
                &[],
                b"ord"
            ])]),
            vec![ParsedEnvelope {
                payload: inscription("text/plain;charset=utf-8", "ord"),
                ..Default::default()
            }],
        );
    }

    #[test]
    fn extract_from_second_input() {
        assert_eq!(
            parse(&[Witness::new(), inscription("foo", [1; 1040]).to_witness()]),
            vec![ParsedEnvelope {
                payload: inscription("foo", [1; 1040]),
                input: 1,
                ..Default::default()
            }]
        );
    }

    #[test]
    fn extract_from_second_envelope() {
        let mut builder = script::Builder::new();
        builder = inscription("foo", [1; 100]).append_reveal_script_to_builder(builder);
        builder = inscription("bar", [1; 100]).append_reveal_script_to_builder(builder);

        assert_eq!(
            parse(&[Witness::from_slice(&[
                builder.into_script().into_bytes(),
                Vec::new()
            ])]),
            vec![
                ParsedEnvelope {
                    payload: inscription("foo", [1; 100]),
                    ..Default::default()
                },
                ParsedEnvelope {
                    payload: inscription("bar", [1; 100]),
                    offset: 1,
                    ..Default::default()
                }
            ]
        );
    }

    #[test]
    fn inscribe_png() {
        assert_eq!(
            parse(&[envelope(&[b"ord", &[1], b"image/png", &[], &[1; 100]])]),
            vec![ParsedEnvelope {
                payload: inscription("image/png", [1; 100]),
                ..Default::default()
            }]
        );
    }

    #[test]
    fn chunked_data_is_parsable() {
        let mut witness = Witness::new();

        witness.push(&inscription("foo", [1; 1040]).append_reveal_script(script::Builder::new()));

        witness.push([]);

        assert_eq!(
            parse(&[witness]),
            vec![ParsedEnvelope {
                payload: inscription("foo", [1; 1040]),
                ..Default::default()
            }]
        );
    }

    #[test]
    fn round_trip_with_no_fields() {
        let mut witness = Witness::new();

        witness.push(Inscription::default().append_reveal_script(script::Builder::new()));

        witness.push([]);

        assert_eq!(
            parse(&[witness]),
            vec![ParsedEnvelope {
                payload: Inscription::default(),
                ..Default::default()
            }],
        );
    }

    #[test]
    fn unknown_odd_fields_are_ignored() {
        assert_eq!(
            parse(&[envelope(&[b"ord", &[11], &[0]])]),
            vec![ParsedEnvelope {
                payload: Inscription::default(),
                ..Default::default()
            }],
        );
    }

    #[test]
    fn unknown_even_fields() {
        assert_eq!(
            parse(&[envelope(&[b"ord", &[22], &[0]])]),
            vec![ParsedEnvelope {
                payload: Inscription {
                    unrecognized_even_field: true,
                    ..Default::default()
                },
                ..Default::default()
            }],
        );
    }

    #[test]
    fn pointer_field_is_recognized() {
        assert_eq!(
            parse(&[envelope(&[b"ord", &[2], &[1]])]),
            vec![ParsedEnvelope {
                payload: Inscription {
                    pointer: Some(vec![1]),
                    ..Default::default()
                },
                ..Default::default()
            }],
        );
    }

    #[test]
    fn duplicate_pointer_field_makes_inscription_unbound() {
        assert_eq!(
            parse(&[envelope(&[b"ord", &[2], &[1], &[2], &[0]])]),
            vec![ParsedEnvelope {
                payload: Inscription {
                    pointer: Some(vec![1]),
                    duplicate_field: true,
                    unrecognized_even_field: true,
                    ..Default::default()
                },
                ..Default::default()
            }],
        );
    }

    #[test]
    fn incomplete_field() {
        assert_eq!(
            parse(&[envelope(&[b"ord", &[99]])]),
            vec![ParsedEnvelope {
                payload: Inscription {
                    incomplete_field: true,
                    ..Default::default()
                },
                ..Default::default()
            }],
        );
    }

    #[test]
    fn metadata_is_parsed_correctly() {
        assert_eq!(
            parse(&[envelope(&[b"ord", &[5], &[]])]),
            vec![ParsedEnvelope {
                payload: Inscription {
                    metadata: Some(vec![]),
                    ..Default::default()
                },
                ..Default::default()
            }]
        );
    }

    #[test]
    fn metadata_is_parsed_correctly_from_chunks() {
        assert_eq!(
            parse(&[envelope(&[b"ord", &[5], &[0], &[5], &[1]])]),
            vec![ParsedEnvelope {
                payload: Inscription {
                    metadata: Some(vec![0, 1]),
                    duplicate_field: true,
                    ..Default::default()
                },
                ..Default::default()
            }]
        );
    }

    #[test]
    fn pushnum_opcodes_are_parsed_correctly() {
        const PUSHNUMS: &[(opcodes::Opcode, u8)] = &[
            (opcodes::all::OP_PUSHNUM_NEG1, 0x81),
            (opcodes::all::OP_PUSHNUM_1, 1),
            (opcodes::all::OP_PUSHNUM_2, 2),
            (opcodes::all::OP_PUSHNUM_3, 3),
            (opcodes::all::OP_PUSHNUM_4, 4),
            (opcodes::all::OP_PUSHNUM_5, 5),
            (opcodes::all::OP_PUSHNUM_6, 6),
            (opcodes::all::OP_PUSHNUM_7, 7),
            (opcodes::all::OP_PUSHNUM_8, 8),
            (opcodes::all::OP_PUSHNUM_9, 9),
            (opcodes::all::OP_PUSHNUM_10, 10),
            (opcodes::all::OP_PUSHNUM_11, 11),
            (opcodes::all::OP_PUSHNUM_12, 12),
            (opcodes::all::OP_PUSHNUM_13, 13),
            (opcodes::all::OP_PUSHNUM_14, 14),
            (opcodes::all::OP_PUSHNUM_15, 15),
            (opcodes::all::OP_PUSHNUM_16, 16),
        ];

        for &(op, value) in PUSHNUMS {
            let script = script::Builder::new()
                .push_opcode(opcodes::OP_FALSE)
                .push_opcode(opcodes::all::OP_IF)
                .push_slice(b"ord")
                .push_opcode(opcodes::OP_FALSE)
                .push_opcode(op)
                .push_opcode(opcodes::all::OP_ENDIF)
                .into_script();

            assert_eq!(
                parse(&[Witness::from_slice(&[script.into_bytes(), Vec::new()])]),
                vec![ParsedEnvelope {
                    payload: Inscription {
                        body: Some(vec![value]),
                        ..Default::default()
                    },
                    pushnum: true,
                    ..Default::default()
                }],
            );
        }
    }
}
