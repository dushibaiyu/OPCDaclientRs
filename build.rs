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
    // 检查目标平台：OPC DA 仅支持 Windows
    // 但在生成文档时跳过此检查
    if !cfg!(target_os = "windows") {
        // 如果是文档生成模式，跳过平台检查
        if env::var("DOCS_RS").is_ok() || env::var("CARGO_DOC").is_ok() {
            println!("cargo:warning=Documentation build on non-Windows platform");
            return Ok(());
        }
        anyhow::bail!("opc-ffi only supports Windows");
    }

    // 确定目标架构
    let arch = if cfg!(target_arch = "x86") {
        "x86"  // 32位 Windows
    } else if cfg!(target_arch = "x86_64") {
        "x64"  // 64位 Windows
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
