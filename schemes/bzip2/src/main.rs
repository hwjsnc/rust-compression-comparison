use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};
use std::io::Read as _;

struct Bzip2 {
    compression: bzip2::Compression,
}

impl DescribeScheme for Bzip2 {
    fn name(&self) -> String {
        "bzip2".to_string()
    }
    fn settings(&self) -> Option<String> {
        Some(format!("level {}", self.compression.level()))
    }
}

impl Compressor for Bzip2 {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        let mut encoder = bzip2::read::BzEncoder::new(data, self.compression);
        let mut vec = vec![];
        encoder
            .read_to_end(&mut vec)
            .context("bzip2 compression error")?;
        Ok(vec)
    }
}

impl Decompressor for Bzip2 {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        let mut decoder = bzip2::read::MultiBzDecoder::new(src);
        decoder
            .read_exact(dst)
            .context("bzip2 decompression error")?;
        let mut tmp = [0u8];
        match decoder.read_exact(&mut tmp) {
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(()),
            _ => Err(anyhow::Error::msg(
                "bzip2 decompression error: dst too short",
            )),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut schemes = vec![];
    for level in 1..=9 {
        schemes.push(Bzip2 {
            compression: bzip2::Compression::new(level),
        });
    }
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
