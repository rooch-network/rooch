// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
// Code from https://github.com/ordinals/ord/

use anyhow::{anyhow, Error};
use brotli::enc::backward_references::BrotliEncoderMode::{
    self, BROTLI_MODE_FONT, BROTLI_MODE_GENERIC, BROTLI_MODE_TEXT,
};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Media {
    Audio,
    Code(Language),
    Font,
    Iframe,
    Image,
    Markdown,
    Model,
    Pdf,
    Text,
    Unknown,
    Video,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Language {
    Css,
    JavaScript,
    Json,
    Python,
    Yaml,
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Css => "css",
                Self::JavaScript => "javascript",
                Self::Json => "json",
                Self::Python => "python",
                Self::Yaml => "yaml",
            }
        )
    }
}

impl Media {
    #[rustfmt::skip]
    const TABLE: &'static [(&'static str, BrotliEncoderMode, Media, &'static [&'static str])] = &[
    ("application/cbor",            BROTLI_MODE_GENERIC, Media::Unknown,                    &["cbor"]),
    ("application/json",            BROTLI_MODE_TEXT,    Media::Code(Language::Json),       &["json"]),
    ("application/octet-stream",    BROTLI_MODE_GENERIC, Media::Unknown,                    &["bin"]),
    ("application/pdf",             BROTLI_MODE_GENERIC, Media::Pdf,                        &["pdf"]),
    ("application/pgp-signature",   BROTLI_MODE_TEXT,    Media::Text,                       &["asc"]),
    ("application/protobuf",        BROTLI_MODE_GENERIC, Media::Unknown,                    &["binpb"]),
    ("application/x-javascript",    BROTLI_MODE_TEXT,    Media::Code(Language::JavaScript), &[]),
    ("application/yaml",            BROTLI_MODE_TEXT,    Media::Code(Language::Yaml),       &["yaml", "yml"]),
    ("audio/flac",                  BROTLI_MODE_GENERIC, Media::Audio,                      &["flac"]),
    ("audio/mpeg",                  BROTLI_MODE_GENERIC, Media::Audio,                      &["mp3"]),
    ("audio/wav",                   BROTLI_MODE_GENERIC, Media::Audio,                      &["wav"]),
    ("font/otf",                    BROTLI_MODE_GENERIC, Media::Font,                       &["otf"]),
    ("font/ttf",                    BROTLI_MODE_GENERIC, Media::Font,                       &["ttf"]),
    ("font/woff",                   BROTLI_MODE_GENERIC, Media::Font,                       &["woff"]),
    ("font/woff2",                  BROTLI_MODE_FONT,    Media::Font,                       &["woff2"]),
    ("image/apng",                  BROTLI_MODE_GENERIC, Media::Image,                      &["apng"]),
    ("image/avif",                  BROTLI_MODE_GENERIC, Media::Image,                      &[]),
    ("image/gif",                   BROTLI_MODE_GENERIC, Media::Image,                      &["gif"]),
    ("image/jpeg",                  BROTLI_MODE_GENERIC, Media::Image,                      &["jpg", "jpeg"]),
    ("image/png",                   BROTLI_MODE_GENERIC, Media::Image,                      &["png"]),
    ("image/svg+xml",               BROTLI_MODE_TEXT,    Media::Iframe,                     &["svg"]),
    ("image/webp",                  BROTLI_MODE_GENERIC, Media::Image,                      &["webp"]),
    ("model/gltf+json",             BROTLI_MODE_TEXT,    Media::Model,                      &["gltf"]),
    ("model/gltf-binary",           BROTLI_MODE_GENERIC, Media::Model,                      &["glb"]),
    ("model/stl",                   BROTLI_MODE_GENERIC, Media::Unknown,                    &["stl"]),
    ("text/css",                    BROTLI_MODE_TEXT,    Media::Code(Language::Css),        &["css"]),
    ("text/html",                   BROTLI_MODE_TEXT,    Media::Iframe,                     &[]),
    ("text/html;charset=utf-8",     BROTLI_MODE_TEXT,    Media::Iframe,                     &["html"]),
    ("text/javascript",             BROTLI_MODE_TEXT,    Media::Code(Language::JavaScript), &["js"]),
    ("text/markdown",               BROTLI_MODE_TEXT,    Media::Markdown,                   &[]),
    ("text/markdown;charset=utf-8", BROTLI_MODE_TEXT,    Media::Markdown,                   &["md"]),
    ("text/plain",                  BROTLI_MODE_TEXT,    Media::Text,                       &[]),
    ("text/plain;charset=utf-8",    BROTLI_MODE_TEXT,    Media::Text,                       &["txt"]),
    ("text/x-python",               BROTLI_MODE_TEXT,    Media::Code(Language::Python),     &["py"]),
    ("video/mp4",                   BROTLI_MODE_GENERIC, Media::Video,                      &["mp4"]),
    ("video/webm",                  BROTLI_MODE_GENERIC, Media::Video,                      &["webm"]),
  ];
}

impl FromStr for Media {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for entry in Self::TABLE {
            if entry.0 == s {
                return Ok(entry.2);
            }
        }

        Err(anyhow!("unknown content type: {s}"))
    }
}
