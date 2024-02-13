extern crate proc_macro;
use std::{env, path::Path, process::Command};

use proc_macro::TokenStream;

const CHANNEL: &str = "nightly-2023-09-30";

// I'm starting to rethink this idea. Even with it working perfectly somehow, you would still probably need a seperate cargo crate to get proper syntax highlighting and stuff. Might give up.
#[proc_macro]
pub fn wow(item: TokenStream) -> TokenStream {
    // got a better plan: Copy file into a cargo crate that is pre setup, then we just compile with cargo, and it is all good.
    let rust_file = item.to_string();
    // This doesn't allow relative paths. Very bad. This should be relative to crate root it is called from?
    match Path::new(&rust_file).try_exists() {
        Ok(true) => (),
        Ok(false) => panic!("File '{}' does not exist.", rust_file.clone()),
        Err(err) => panic!("{}", err),
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();

    // As far as I can tell, it is not possible to both create a binary and work out the crate name at the same time.
    let get_crate_name = Command::new("rustc")
        .current_dir(out_dir.clone())
        .env_remove("CARGO_MAKEFLAGS")
        .arg(rust_file.clone())
        .arg("--print=crate-name")
        .output()
        .unwrap();
    let mut crate_name = String::from_utf8(get_crate_name.stdout).unwrap();
    crate_name.pop().unwrap();

    let compile = Command::new("rustup")
        .current_dir(out_dir.clone())
        .env_remove("CARGO_MAKEFLAGS")
        .arg("run")
        .arg(CHANNEL)
        .arg("rustc")
        .arg(rust_file)
        .output()
        .unwrap();

    // compile must succeed or else it could run an old binary still in OUT_DIR
    if !compile.status.success() {
        let compile_stderr = String::from_utf8(compile.stderr).unwrap();
        panic!("{}", compile_stderr);
    }

    let run = Command::new(format!("./{}", crate_name))
        .current_dir(out_dir)
        .output()
        .expect(format!("./{}", crate_name).as_str());

    // Temp, please remove.
    format!(
        "\"compile: {}, {}, {}\nrun: {}, {}, {}\"",
        compile.status,
        String::from_utf8(compile.stderr).unwrap(),
        crate_name,
        run.status,
        String::from_utf8(run.stderr).unwrap(),
        String::from_utf8(run.stdout).unwrap()
    )
    .parse()
    .unwrap()
}
