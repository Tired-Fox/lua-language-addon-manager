use std::path::PathBuf;

use luarc::{Error, LuaRc};

fn main() -> Result<(), Error> {
    let rc = LuaRc::detect(PathBuf::from("examples/basicrc.json"))?;
    println!("{:#?}", rc);

    Ok(())
}
