use anyhow::{bail, Result};
use clap::Parser;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

mod usdt {
    include!(concat!(env!("OUT_DIR"), "/usdt.skel.rs"));
}
use usdt::*;

#[derive(Parser)]
struct Cli {
    #[clap(long, value_name = "FILE")]
    program: PathBuf,
}

fn bump_memlock_rlimit() -> Result<()> {
    let rlimit = libc::rlimit {
        rlim_cur: 128 << 20,
        rlim_max: 128 << 20,
    };

    if unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlimit) } != 0 {
        bail!("Failed to increase rlimit");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    bump_memlock_rlimit()?;

    let skel_builder = UsdtSkelBuilder::default();
    let open_skel = skel_builder.open()?;
    let mut skel = open_skel.load()?;

    let mut progs = skel.progs_mut();
    let prog = progs.usdt__trace();
    let _link = prog.attach_usdt(
        -1, // any process
        &cli.program,
        "provider",
        "function",
    )?;

    let maps = skel.maps();
    let map = maps.ringbuf();

    let mut rb = libbpf_async::RingBuffer::new(skel.obj.map_mut("ringbuf").unwrap());

    // Call getpid to ensure the BPF program runs
    unsafe { libc::getpid() };

    loop {
        let mut buf = [0; 8];
        let n = rb.read(&mut buf).await.unwrap();
        let value = u64::from_le_bytes(buf.try_into().unwrap());
        println!("callback {}", value);
    }
}
