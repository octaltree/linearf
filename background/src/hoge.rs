#![feature(test)]
extern crate test;

use neovim_lib::{neovim::Neovim, neovim_api::NeovimApi, session::Session};
use std::time::Instant;

fn main() {
    let server = "/tmp/nvimzXhnZe/0";
    let mut session = Session::new_unix_socket(&server).unwrap();
    session.start_event_loop();
    let mut nvim = Neovim::new(session);
    let now = Instant::now();
    let arr = arr();
    write(&mut nvim, arr).unwrap();
    println!("{:?}", Instant::now() - now);
}

fn arr() -> Vec<String> { (1..=500).map(|i| i.to_string()).collect() }

fn write(nvim: &mut Neovim, arr: Vec<String>) -> anyhow::Result<()> {
    let buf = nvim.get_current_buf()?;
    buf.set_lines(nvim, 0, -1, true, arr)?;
    Ok(())
}

// fn write<W>(neovim: &Neovim<W>, arr: Vec<String>) -> anyhow::Result<()>
// where
//    W: futures_io::AsyncWrite + Send + Unpin + 'static
//{
//    let buffer = neovim.get_current_buf()?;
//    buffer.set_lines(0, -1, true, arr)?;
//    Ok(())
//}

#[cfg(test)]
mod tests {
    use super::*;
    use test::bench::Bencher;

    //#[bench]
    // fn bench_arr(b: &mut Bencher) { b.iter(|| arr()) }

    //#[bench]
    // fn bench_main(b: &mut Bencher) { b.iter(|| main()) }

    #[bench]
    fn bench_write(b: &mut Bencher) {
        let server = "/tmp/nvimdrS4KB/0";
        let mut session = Session::new_unix_socket(&server).unwrap();
        session.start_event_loop();
        let mut nvim = Neovim::new(session);
        let arr = arr();
        b.iter(|| write(&mut nvim, arr.clone()))
    }

    //#[bench]
    // fn bench_write(b: &mut Bencher) { b.iter(|| main()) }
}
