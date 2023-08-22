// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;

#[cfg(feature = "dashboard")]
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;

#[cfg(feature = "dashboard")]
use rocket::http::ContentType;
#[cfg(feature = "dashboard")]
use rocket::response::content::RawHtml;
#[cfg(feature = "dashboard")]
use rust_embed::RustEmbed;

#[cfg(feature = "dashboard")]
use std::borrow::Cow;
#[cfg(feature = "dashboard")]
use std::ffi::OsStr;
#[cfg(feature = "dashboard")]
use std::path::PathBuf;

#[cfg(feature = "dashboard")]
#[derive(RustEmbed)]
#[folder = "public/dashboard/"]
struct Asset;

#[cfg(feature = "dashboard")]
#[get("/")]
fn index() -> Option<RawHtml<Cow<'static, [u8]>>> {
    println!("get index -------");
    let asset = Asset::get("index.html")?;
    Some(RawHtml(asset.data))
}

#[cfg(feature = "dashboard")]
#[get("/<file..>")]
fn dist(file: PathBuf) -> Option<(ContentType, Cow<'static, [u8]>)> {
    let mut filename = file.display().to_string();

    if !filename.starts_with("_next")
        && !filename.starts_with("images")
        && !filename.ends_with(".html")
    {
        filename = filename + "/index.html";
        let asset = Asset::get(&filename)?;
        return Some((ContentType::HTML, asset.data));
    }

    let asset = Asset::get(&filename)?;
    let content_type = file
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ContentType::from_extension)
        .unwrap_or(ContentType::Bytes);

    Some((content_type, asset.data))
}

/// Start Rooch Dashboard
#[derive(Parser)]
pub struct Dashboard;

#[cfg(feature = "dashboard")]
#[async_trait]
impl CommandAction<String> for Dashboard {
    async fn execute(self) -> RoochResult<String> {
        let s = rocket::build().mount("/", routes![index, dist]);

        let _ = s.launch().await;

        Ok("Rocket: deorbit.".to_owned())
    }
}

#[cfg(not(feature = "dashboard"))]
#[async_trait]
impl CommandAction<String> for Dashboard {
    async fn execute(self) -> RoochResult<String> {
        Ok("Dashboard feature is not enabled.".to_owned())
    }
}
