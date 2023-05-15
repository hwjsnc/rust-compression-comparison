use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};
use std::io::{Read, Write};

#[derive(Debug)]
enum Lz4 {
    Block,
    Frame,
}

impl DescribeScheme for Lz4 {
    fn name(&self) -> String {
        "lz4_flex".to_string()
    }
    fn settings(&self) -> Option<String> {
        Some(format!("safe / {:?}", self))
    }
}

impl Compressor for Lz4 {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        match self {
            Lz4::Block => Ok(lz4_flex::block::compress(data)),
            Lz4::Frame => {
                let mut compressor = lz4_flex::frame::FrameEncoder::new(vec![]);
                compressor
                    .write_all(data)
                    .context("lz4_flex compression error")?;
                compressor.finish().context("lz4_flex compression error")
            }
        }
    }
}

impl Decompressor for Lz4 {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        match self {
            Lz4::Block => {
                let len = lz4_flex::block::decompress_into(src, dst)
                    .context("lz4_flex decompression error")?;
                anyhow::ensure!(len == dst.len(), "dst buffer length does not match");
                Ok(())
            }
            Lz4::Frame => {
                let mut decompressor = lz4_flex::frame::FrameDecoder::new(src);
                decompressor
                    .read_exact(dst)
                    .context("lz4_flex decompression error")?;
                let mut tmp = [0u8];
                match decompressor.read_exact(&mut tmp) {
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(()),
                    _ => Err(anyhow::Error::msg(
                        "lz4_flex decompression error: dst too short",
                    )),
                }
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let schemes = [Lz4::Block, Lz4::Frame];
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
