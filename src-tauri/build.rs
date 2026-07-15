use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if let Ok(target) = env::var("TARGET") {
        println!("cargo:rustc-env=BUILD_TARGET={target}");
    }

    if let Err(error) = bundle_ffmpeg_binary() {
        println!("cargo:warning=FFmpeg bundle failed: {error}");
        println!(
            "cargo:warning=MP4 conversion will require internet on first app launch."
        );
    }

    tauri_build::build();
}

fn bundle_ffmpeg_binary() -> Result<(), String> {
    std::env::set_var("KEEP_ONLY_FFMPEG", "1");

    let target = env::var("TARGET").map_err(|error| error.to_string())?;
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").map_err(|error| error.to_string())?;
    let bin_dir = PathBuf::from(&manifest_dir).join("binaries");
    fs::create_dir_all(&bin_dir).map_err(|error| error.to_string())?;

    let dest_name = if target.contains("windows") {
        format!("ffmpeg-{target}.exe")
    } else {
        format!("ffmpeg-{target}")
    };
    let dest = bin_dir.join(&dest_name);

    if dest.exists() {
        return Ok(());
    }

    let unpack_dir = bin_dir.join("_ffmpeg_build_unpack");
    if unpack_dir.exists() {
        fs::remove_dir_all(&unpack_dir).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&unpack_dir).map_err(|error| error.to_string())?;

    let url = ffmpeg_sidecar::download::ffmpeg_download_url().map_err(|error| error.to_string())?;
    let archive = ffmpeg_sidecar::download::download_ffmpeg_package(url, &unpack_dir)
        .map_err(|error| error.to_string())?;
    ffmpeg_sidecar::download::unpack_ffmpeg_without_extras(&archive, &unpack_dir)
        .map_err(|error| error.to_string())?;

    let unpacked_ffmpeg = if target.contains("windows") {
        unpack_dir.join("ffmpeg.exe")
    } else {
        unpack_dir.join("ffmpeg")
    };

    if !unpacked_ffmpeg.exists() {
        return Err("ffmpeg binary not found after unpack".to_string());
    }

    fs::copy(&unpacked_ffmpeg, &dest).map_err(|error| error.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&dest)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&dest, permissions).map_err(|error| error.to_string())?;
    }

    fs::remove_dir_all(&unpack_dir).ok();
    if archive.exists() {
        fs::remove_file(&archive).ok();
    }

    Ok(())
}
