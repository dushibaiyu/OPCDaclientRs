//! 构建脚本 (build.rs)
//! 
//! 这个构建脚本负责：
//! 1. 检查目标平台（仅支持 Windows）
//! 2. 确定目标架构（x86 或 x86_64）
//! 3. 设置 OPC 库的链接路径
//! 4. 复制 DLL 文件到输出目录
//! 
//! ## 依赖库
//! 
//! 本库依赖于预编译的 OPCClientToolKit 动态库：
//! - OPCClientToolKit.dll: 动态链接库
//! - OPCClientToolKit.lib: 导入库
//! 
//! 这些库文件位于 `libs/{arch}/` 目录中。
//! 
//! ## 构建过程
//! 
//! 1. 检查平台和架构
//! 2. 设置库搜索路径
//! 3. 指定要链接的库
//! 4. 复制 DLL 到输出目录

use anyhow::{Context, Result};
use std::{env, path::PathBuf};

fn main() -> Result<()> {
    // Check target platform: OPC DA only supports Windows for actual FFI calls
    // Allow compilation on non-Windows for type system development and testing
    if !cfg!(target_os = "windows") {
        // If building documentation, skip platform check
        if env::var("DOCS_RS").is_ok() || env::var("CARGO_DOC").is_ok() {
            println!("cargo:warning=Documentation build on non-Windows platform");
            return Ok(());
        }
        // Allow compilation for type system testing without FFI
        println!("cargo:warning=Compiling on non-Windows platform - FFI functionality will be limited");
        println!("cargo:rustc-cfg=no_ffi");
        return Ok(());
    }

    // Determine target architecture (Windows only)
    let arch = if cfg!(target_arch = "x86") {
        "x86"  // 32-bit Windows
    } else if cfg!(target_arch = "x86_64") {
        "x64"  // 64-bit Windows
    } else {
        anyhow::bail!("unsupported architecture");
    };


    let lib_dir = PathBuf::from("libs").join(arch);
    if !lib_dir.exists() {
        anyhow::bail!("{:?} not found", lib_dir);
    }

    // Tell cargo where to find the .lib file
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    // Tell cargo to link this library
    println!("cargo:rustc-link-lib=dylib=OPCClientToolKit");

    // If DLL is not in PATH, copy it to target/{debug/release} directory
    let dll_name = "OPCClientToolKit.dll";
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    std::fs::copy(lib_dir.join(dll_name), out_dir.join(dll_name))
        .context("copy dll to OUT_DIR")?;
    println!("cargo:rustc-env=PATH={}", out_dir.display());

    Ok(())
}
