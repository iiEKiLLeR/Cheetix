//! Embeds the SM7 icon and version information in sm7.exe (Explorer, taskbar, installer and
//! "Installed apps" all show them). Uses windres from the MinGW toolchain the GNU target needs.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/sm7.ico");
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_WINDOWS").is_err() {
        return;
    }
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ver = std::env::var("CARGO_PKG_VERSION").unwrap();
    let nums: Vec<&str> = ver.split('.').collect();
    let comma = format!("{},{},{},0", nums[0], nums[1], nums[2]);
    let ico = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets").join("sm7.ico");
    let rc = format!(
        r#"1 ICON "{ico}"
1 VERSIONINFO
FILEVERSION {comma}
PRODUCTVERSION {comma}
FILEOS 0x40004
FILETYPE 0x1
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "CompanyName", "SM7"
      VALUE "FileDescription", "SM7"
      VALUE "FileVersion", "{ver}"
      VALUE "InternalName", "sm7"
      VALUE "OriginalFilename", "sm7.exe"
      VALUE "ProductName", "SM7"
      VALUE "ProductVersion", "{ver}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#,
        ico = ico.display().to_string().replace('\\', "/")
    );
    let rc_path = out.join("sm7.rc");
    std::fs::write(&rc_path, rc).unwrap();
    let obj = out.join("sm7_res.o");
    let ok = Command::new("windres").arg("-O").arg("coff").arg("-i").arg(&rc_path).arg("-o").arg(&obj).status().map(|s| s.success()).unwrap_or(false);
    if ok {
        println!("cargo:rustc-link-arg-bins={}", obj.display());
    } else {
        // the app still builds, only without an embedded icon
        println!("cargo:warning=windres not found or failed: sm7.exe will have no icon");
    }
}
