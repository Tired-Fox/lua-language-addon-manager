use std::{collections::BTreeMap, str::FromStr};

use clap::{ArgAction, Parser, Subcommand};
use llam::{Addon, Error};
use luarc::LuaRc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(long, short, action = ArgAction::SetTrue)]
    global: bool
}

#[derive(Debug, Subcommand)]
enum Command {
    Add {
        name: String
    }
}


#[derive(Default, Debug, Deserialize, Serialize)]
struct Workspace {
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    addons: BTreeMap<String, Addon>
}

fn main() -> Result<(), Error> {
    let base = std::env::current_dir()?;
    let addons = base.join(".addons");
    let luarc = base.join(".luarc.json");

    let mut rc = LuaRc::extend()
        .workspace::<Workspace>()
        .read(&luarc)?
        .ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, ".luarc.json file not found"))?;

    let cli = Cli::parse();
    match cli.command {
        Command::Add { name } => {
            let mut addon = Addon::from_str(&name)?;

            if addon.exists(&addons) {
                println!("Pulling addon's latest version at .addons/{}", &addon.repo);
                addon.pull(&addons)?;
            } else {
                println!("Downloading addon to .addons/{}", &addon.repo);
                addon.download(&addons)?;
            }

            if rc.workspace.is_none() {
                rc.workspace = Some(Default::default());
            }

            let workspace = rc.workspace.as_mut().unwrap();

            println!("Updating .luarc.json");
            _ = workspace.addons.insert(addon.repo.clone(), addon);
            let path = dunce::canonicalize(&addons)?.display().to_string();
            if !workspace.user_third_party.iter().any(|v| v.as_ref() == ".addons" || v.as_ref() == path) {
                workspace.user_third_party.push(".addons".into());
            }
            rc.write(&luarc)?;
        }
    }

    Ok(())
}
