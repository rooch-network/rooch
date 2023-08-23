// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::{
    env, fs,
    path::Path,
    process::{self, Command},
};

fn main() {
    // build dashboard
    if cfg!(feature = "dashboard") {
        let base_path: String;
        let dashboard_dir = "dashboard";
        let out_put_dir = "crates/rooch/public/dashboard/";

        if let Ok(output) = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
        {
            base_path = String::from_utf8_lossy(&output.stdout).trim().to_string();

            println!("cargo:rerun-if-changed={}/{}", base_path, dashboard_dir);

            let dashboard_path = Path::new(&base_path).join(dashboard_dir);
            env::set_current_dir(dashboard_path.clone()).unwrap();

            let npm_status = Command::new("npm").args(["install", "-g", "yarn"]).status();

            if npm_status.is_err() {
                eprintln!("yarn install failed");
                process::exit(1);
            }

            let yarn_status = Command::new("yarn").status();

            if yarn_status.is_err() {
                eprintln!("yarn install failed");
                process::exit(1);
            }

            let export_status = Command::new("yarn").args(["export"]).status();

            if let Ok(status) = export_status {
                if status.success() {
                    let out_dir = dashboard_path.join("out");
                    let destination_dir = Path::new(&base_path).join(out_put_dir);
                    println!("{:?}", destination_dir);
                    if let Err(err) = copy_directory(&out_dir, &destination_dir) {
                        eprintln!("Failed to copy directory: {}", err);
                        process::exit(1);
                    }
                } else {
                    eprintln!("yarn build failed");
                    process::exit(1);
                }
            } else {
                eprintln!("yarn build failed");
                process::exit(1);
            }
        }
    }
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    if !source.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Source directory does not exist",
        ));
    }

    if !destination.exists() {
        fs::create_dir_all(destination)?;
    }

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if source_path.is_dir() {
            copy_directory(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)?;
        }
    }

    Ok(())
}
