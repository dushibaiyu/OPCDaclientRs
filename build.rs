use anyhow::{Context, Result};
use std::{env, path::PathBuf};

fn main() -> Result<()> {
    if !cfg!(target_os = "windows") {
        anyhow::bail!("opc-ffi only supports Windows");
    }

    // build.rs
    let arch = if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "x86_64") {
        "x64"
    } else {
        anyhow::bail!("unsupported architecture");
    };


    let lib_dir = PathBuf::from("libs").join(arch);
    if !lib_dir.exists() {
        anyhow::bail!("{:?} not found", lib_dir);
    }

    // 告诉 cargo 去这里找 .lib
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    // 告诉 cargo 要链接的库名字（去掉前缀和后缀）
    println!("cargo:rustc-link-lib=dylib=OPCClientToolKit");

    // 若 DLL 不在 PATH，可把它拷贝到 target/{debug/release} 目录
    let dll_name = "OPCClientToolKit.dll";
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    std::fs::copy(lib_dir.join(dll_name), out_dir.join(dll_name))
        .context("copy dll to OUT_DIR")?;
    println!("cargo:rustc-env=PATH={}", out_dir.display());

    Ok(())
}
