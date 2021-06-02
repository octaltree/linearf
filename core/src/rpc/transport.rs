use anyhow::Context as _;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    convert::TryInto,
    fmt::Debug,
    io::{stdin, Read, Stdin},
    stream::Stream
};
use tokio::io::{stdout, AsyncWriteExt, Stdout};

#[derive(Debug)]
pub(super) struct Reader {
    stdin: Stdin,
    length: Option<u32>,
    buf: Vec<u8>
}

#[derive(Debug)]
pub(super) struct Writer {
    stdout: Stdout
}

pub enum ReadError {
    Closed,
    Error(anyhow::Error)
}

impl Reader {
    const BUFSIZE: usize = 30000;

    pub(super) fn new() -> Self {
        let stdin = stdin();
        Self {
            stdin,
            length: None,
            buf: Vec::with_capacity(Self::BUFSIZE)
        }
    }

    // TODO: heap efficiency
    pub(super) fn read<M>(&mut self) -> Result<Option<M>, ReadError>
    where
        M: DeserializeOwned
    {
        let this = self;
        {
            if this.length.is_none() && this.buf.len() >= 4 {
                let off = this.buf.split_off(4);
                let bytes: &[u8] = &this.buf;
                this.length = Some(u32::from_le_bytes(bytes.try_into().unwrap()));
                this.buf = off;
            }
            match this.length.map(|u| u as usize) {
                None => {}
                Some(l) if this.buf.len() < l => {}
                Some(l) => {
                    let bytes: &[u8] = &this.buf[..l];
                    log::debug!("RECV {}", unsafe { std::str::from_utf8_unchecked(bytes) });
                    let msg: M =
                        serde_json::from_slice(bytes).map_err(|e| ReadError::Error(e.into()))?;
                    this.length = None;
                    this.buf = this.buf[l..].to_owned();
                    return Ok(Some(msg));
                }
            }
        }
        {
            let mut buf = [0; Self::BUFSIZE];
            let n = this
                .stdin
                .read(&mut buf)
                .map_err(|e| ReadError::Error(e.into()))?;
            if n == 0 {
                return Err(ReadError::Closed);
            }
            this.buf.extend(&buf[..n]);
        }
        Ok(None)
    }
}

impl Writer {
    pub(super) fn new() -> Self {
        let stdout = stdout();
        Self { stdout }
    }

    pub(super) async fn send<M>(&mut self, msg: &M) -> anyhow::Result<()>
    where
        M: Serialize + Debug
    {
        log::debug!("SEND {:?}", msg);
        let serialized = serde_json::to_vec(msg).with_context(|| format!("{:?}", msg))?;
        let length = serialized.len() as u32;
        let mut bytes = length.to_le_bytes().to_vec();
        bytes.extend(serialized);
        self.stdout.write_all(&bytes).await?;
        Ok(())
    }
}
