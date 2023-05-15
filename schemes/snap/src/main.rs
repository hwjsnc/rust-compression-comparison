use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};
use std::io::{Read, Write};

#[derive(Debug)]
enum Snap {
    Raw,
    Frame,
}

impl DescribeScheme for Snap {
    fn name(&self) -> String {
        "snap".to_string()
    }
    fn settings(&self) -> Option<String> {
        Some(format!("{self:?}"))
    }
}

impl Compressor for Snap {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        match self {
            Snap::Raw => snap::raw::Encoder::new()
                .compress_vec(data)
                .context("snappy compression failed"),
            Snap::Frame => {
                let mut encoder = snap::write::FrameEncoder::new(vec![]);
                encoder
                    .write_all(data)
                    .context("snappy compression failed")?;
                encoder.into_inner().context("snappy compression failed")
            }
        }
    }
}

impl Decompressor for Snap {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        match self {
            Snap::Raw => {
                let len = snap::raw::Decoder::new()
                    .decompress(src, dst)
                    .context("snappy decompression failed")?;
                anyhow::ensure!(len == dst.len(), "snappy decompression error: dst too long");
                Ok(())
            }
            Snap::Frame => {
                let mut decoder = snap::read::FrameDecoder::new(src);
                decoder
                    .read_exact(dst)
                    .context("lzma decompression error")?;
                let mut tmp = [0u8];
                match decoder.read_exact(&mut tmp) {
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(()),
                    _ => Err(anyhow::Error::msg(
                        "snap decompression failed: dst too short",
                    )),
                }
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let schemes = [Snap::Raw, Snap::Frame];
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
