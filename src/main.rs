use anyhow::{bail, Result};
use clap::Parser;
use libbpf_rs::RingBufferBuilder;
use std::path::PathBuf;
use std::time::Duration;

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

fn callback(data: &[u8]) -> i32 {
    let value = u64::from_le_bytes(data.try_into().unwrap());
    println!("callback {}", value);
    0
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    bump_memlock_rlimit()?;

    let skel_builder = UsdtSkelBuilder::default();
    let open_skel = skel_builder.open()?;
    let mut skel = open_skel.load()?;

    skel.attach()?;

    // let mut progs = skel.progs_mut();
    // let prog = progs.usdt__trace();
    // let _link = prog.attach_usdt(
    //     0, &cli.program, "provider", "function",
    // )?;

    let mut builder = RingBufferBuilder::new();
    let maps = skel.maps();
    let map = maps.ringbuf();

    builder.add(map, callback)?;

    let mgr = builder.build()?;

    // Call getpid to ensure the BPF program runs
    unsafe { libc::getpid() };

    mgr.consume()?;
    loop {
        mgr.poll(Duration::from_millis(100))?;
    }
}
