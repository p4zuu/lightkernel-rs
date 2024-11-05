use std::{env::current_dir, fs, path::Path};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, short, default_value_t = false)]
    test: bool,

    #[arg(long, short, default_value_t = true)]
    uefi: bool,
}

fn main() {
    let args = Args::parse();

    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");

    if args.test {
        let mut cmd = std::process::Command::new("qemu-system-x86_64");
        if args.uefi {
            cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
            cmd.arg("-drive")
                .arg(format!("format=raw,file={uefi_path}"));
        } else {
            cmd.arg("-drive")
                .arg(format!("format=raw,file={bios_path}"));
        }
        cmd.args(["-nographic", "-cpu", "host", "-enable-kvm"]);
        let mut child = cmd.spawn().unwrap();
        child.wait().unwrap();
    }

    let src = Path::new(uefi_path);
    let dst = current_dir()
        .unwrap()
        .join(Path::new(uefi_path).file_name().unwrap());

    fs::copy(src, dst).expect("failed to copy the uefi file");
}
