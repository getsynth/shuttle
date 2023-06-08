use std::{
    fs::{read_to_string, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use cargo_generate::{GenerateArgs, TemplatePath, Vcs};
use indoc::indoc;
use shuttle_common::project::ProjectName;
use toml_edit::{value, Document};

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Template {
    ActixWeb,
    Axum,
    Poise,
    Poem,
    Rocket,
    Salvo,
    Serenity,
    Tide,
    Thruster,
    Tower,
    Warp,
    None,
}

impl Template {
    /// Returns a framework-specific struct that implements the trait `ShuttleInit`
    /// for writing framework-specific dependencies to `Cargo.toml` and generating
    /// boilerplate code in `src/main.rs`.
    pub fn init_config(&self) -> Box<dyn ShuttleInit> {
        use Template::*;
        match self {
            ActixWeb => Box::new(ShuttleInitActixWeb),
            Axum => Box::new(ShuttleInitAxum),
            Rocket => Box::new(ShuttleInitRocket),
            Tide => Box::new(ShuttleInitTide),
            Tower => Box::new(ShuttleInitTower),
            Poem => Box::new(ShuttleInitPoem),
            Salvo => Box::new(ShuttleInitSalvo),
            Serenity => Box::new(ShuttleInitSerenity),
            Poise => Box::new(ShuttleInitPoise),
            Warp => Box::new(ShuttleInitWarp),
            Thruster => Box::new(ShuttleInitThruster),
            None => Box::new(ShuttleInitNoOp),
        }
    }
}

pub trait ShuttleInit {
    fn get_repo_url(&self) -> &'static str;
    fn get_sub_path(&self) -> Option<&'static str>;
}

pub struct ShuttleInitActixWeb;

impl ShuttleInit for ShuttleInitActixWeb {
    fn get_repo_url(&self) -> &'static str {
        "http://github.com/shuttle-hq/shuttle-examples.git"
    }

    fn get_sub_path(&self) -> Option<&'static str> {
        Some("actix-web/hello-world")
    }
}

pub struct ShuttleInitAxum;

impl ShuttleInit for ShuttleInitAxum {
    fn get_repo_url(&self) -> &'static str {
        "http://github.com/shuttle-hq/shuttle-examples.git"
    }

    fn get_sub_path(&self) -> Option<&'static str> {
        Some("axum/hello-world")
    }
}

pub struct ShuttleInitRocket;

impl ShuttleInit for ShuttleInitRocket {
    fn get_repo_url(&self) -> &'static str {
        "http://github.com/shuttle-hq/shuttle-examples.git"
    }

    fn get_sub_path(&self) -> Option<&'static str> {
        Some("rocket/hello-world")
    }
}

pub struct ShuttleInitTide;

impl ShuttleInit for ShuttleInitTide {
    fn get_repo_url(&self) -> &'static str {
        "http://github.com/shuttle-hq/shuttle-examples.git"
    }

    fn get_sub_path(&self) -> Option<&'static str> {
        Some("tide/hello-world")
    }
}

pub struct ShuttleInitPoem;

impl ShuttleInit for ShuttleInitPoem {
    fn get_repo_url(&self) -> &'static str {
        "http://github.com/shuttle-hq/shuttle-examples.git"
    }

    fn get_sub_path(&self) -> Option<&'static str> {
        Some("poem/hello-world")
    }
}

pub struct ShuttleInitSalvo;

impl ShuttleInit for ShuttleInitSalvo {
    fn get_repo_url(&self) -> &'static str {
        "http://github.com/shuttle-hq/shuttle-examples.git"
    }

    fn get_sub_path(&self) -> Option<&'static str> {
        Some("salvo/hello-world")
    }
}

pub struct ShuttleInitSerenity;

impl ShuttleInit for ShuttleInitSerenity {
    fn get_repo_url(&self) -> &'static str {
        "http://github.com/shuttle-hq/shuttle-examples.git"
    }

    fn get_sub_path(&self) -> Option<&'static str> {
        Some("serenity/hello-world")
    }
}

pub struct ShuttleInitPoise;

impl ShuttleInit for ShuttleInitPoise {
    fn get_repo_url(&self) -> &'static str {
        "http://github.com/shuttle-hq/shuttle-examples.git"
    }

    fn get_sub_path(&self) -> Option<&'static str> {
        Some("poise/hello-world")
    }
}

pub struct ShuttleInitTower;

impl ShuttleInit for ShuttleInitTower {
    fn get_repo_url(&self) -> &'static str {
        "http://github.com/shuttle-hq/shuttle-examples.git"
    }

    fn get_sub_path(&self) -> Option<&'static str> {
        Some("tower/hello-world")
    }
}

pub struct ShuttleInitWarp;

impl ShuttleInit for ShuttleInitWarp {
    fn get_repo_url(&self) -> &'static str {
        "http://github.com/shuttle-hq/shuttle-examples.git"
    }

    fn get_sub_path(&self) -> Option<&'static str> {
        Some("warp/hello-world")
    }
}

pub struct ShuttleInitThruster;

impl ShuttleInit for ShuttleInitThruster {
    fn get_repo_url(&self) -> &'static str {
        "http://github.com/shuttle-hq/shuttle-examples.git"
    }

    fn get_sub_path(&self) -> Option<&'static str> {
        Some("thruster/hello-world")
    }
}

pub struct ShuttleInitNoOp;
impl ShuttleInit for ShuttleInitNoOp {
    fn get_repo_url(&self) -> &'static str {
        "http://github.com/shuttle-hq/shuttle-examples.git"
    }

    fn get_sub_path(&self) -> Option<&'static str> {
        Some("custom/none")
    }
}

pub fn cargo_generate(path: PathBuf, name: &ProjectName, framework: Template) -> Result<()> {
    let config = framework.init_config();

    println!(r#"    Creating project "{name}" in {path:?}"#);
    let generate_args = GenerateArgs {
        init: true,
        template_path: TemplatePath {
            git: Some(config.get_repo_url().to_string()),
            auto_path: config.get_sub_path().map(str::to_string),
            ..Default::default()
        },
        name: Some(name.to_string()), // appears to do nothing...
        destination: Some(path.clone()),
        vcs: Some(Vcs::Git),
        ..Default::default()
    };
    cargo_generate::generate(generate_args)
        .with_context(|| "Failed to initialize with cargo generate.")?;

    set_crate_name(&path, name.as_str()).with_context(|| "Failed to set crate name.")?;
    remove_shuttle_toml(&path);
    create_gitignore_file(&path).with_context(|| "Failed to create .gitignore file.")?;

    Ok(())
}

// since I can't get cargo-generate to do this for me...
fn set_crate_name(path: &Path, name: &str) -> Result<()> {
    // read the Cargo.toml file
    let mut path = path.to_path_buf();
    path.push("Cargo.toml");

    let toml_str = read_to_string(&path)?;
    let mut doc = toml_str.parse::<Document>()?;

    // change the name
    doc["package"]["name"] = value(name);

    // write the Cargo.toml file back out
    std::fs::write(&path, doc.to_string())?;

    Ok(())
}

/*
Currently Shuttle.toml only has a project name override.
This project name will already be in use, so the file is useless.

If we start putting more things in Shuttle.toml we may wish to re-evaluate.
*/
fn remove_shuttle_toml(path: &Path) {
    let mut path = path.to_path_buf();
    path.push("Shuttle.toml");

    // this file only exists for some of the examples, it's fine if we don't find it
    _ = std::fs::remove_file(path);
}

fn create_gitignore_file(path: &Path) -> Result<()> {
    let mut path = path.to_path_buf();
    path.push(".gitignore");

    let mut file = match OpenOptions::new().create_new(true).write(true).open(path) {
        Ok(f) => f,
        Err(e) => {
            match e.kind() {
                ErrorKind::AlreadyExists => {
                    // if the example already has a .gitignore file, just use that
                    return Ok(());
                }
                _ => {
                    return Err(anyhow!(e));
                }
            }
        }
    };

    file.write_all(indoc! {b"
        /target
        Secrets*.toml
    "})?;

    Ok(())
}
