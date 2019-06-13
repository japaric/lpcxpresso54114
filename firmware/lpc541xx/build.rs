use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    const MASTER: &str = "thumbv7em-none-eabihf";
    #[allow(dead_code)]
    const SLAVE: &str = "thumbv6m-none-eabi";

    let target = env::var("TARGET")?;
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let name = env::var("CARGO_PKG_NAME")?;

    if target == MASTER {
        println!("cargo:rustc-cfg=master");

        fs::copy(
            format!("bin/{}.a", target),
            out_dir.join(format!("lib{}.a", name)),
        )?;

        println!("cargo:rustc-link-lib=static={}", name);
        println!("cargo:rustc-link-search={}", out_dir.display());
    }

    Ok(())
}
