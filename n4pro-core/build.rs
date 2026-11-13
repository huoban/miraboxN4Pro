// 构建脚本：调用 Windows SDK 的 rc.exe，把 assets/app.ico 嵌入 exe
// 资源组 ID 固定为 1，托盘按 ID=1 加载图标

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// 查找 rc.exe：优先环境变量 RC，其次 Windows Kits 最新版本
fn find_rc() -> Option<PathBuf> {
    if let Ok(path) = env::var("RC") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }
    let kits = env::var("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(r"C:\Program Files (x86)"));
    let base = kits.join("Windows Kits").join("10").join("bin");
    let mut found: Vec<PathBuf> = Vec::new();
    if let Ok(rd) = fs::read_dir(&base) {
        for entry in rd.flatten() {
            let candidate = entry.path().join("x64").join("rc.exe");
            if candidate.exists() {
                found.push(candidate);
            }
        }
    }
    found.sort();
    found.pop()
}

fn main() {
    println!("cargo:rerun-if-changed=assets/app.ico");
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ico = root.join("assets").join("app.ico");
    if !ico.exists() {
        println!("cargo:warning=未找到 assets/app.ico，跳过图标嵌入");
        return;
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let rc_file = out_dir.join("app_icon.rc");
    let res_file = out_dir.join("app_icon.res");
    let ico_text = ico.to_string_lossy().replace('\\', "/");
    if fs::write(&rc_file, format!("1 ICON \"{}\"\n", ico_text)).is_err() {
        println!("cargo:warning=生成 .rc 失败，跳过图标嵌入");
        return;
    }

    let Some(rc) = find_rc() else {
        println!("cargo:warning=未找到 rc.exe，跳过图标嵌入（不影响运行）");
        return;
    };
    match Command::new(&rc)
        .arg("/nologo")
        .arg("/fo")
        .arg(&res_file)
        .arg(&rc_file)
        .status()
    {
        Ok(status) if status.success() => {
            println!("cargo:rustc-link-arg={}", res_file.display());
        }
        Ok(status) => println!("cargo:warning=rc.exe 失败：{}", status),
        Err(e) => println!("cargo:warning=调用 rc.exe 失败：{}", e),
    }
}
