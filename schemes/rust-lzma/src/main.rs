use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};
use std::io::Read;

struct Lzma {
    preset: u32,
}

impl DescribeScheme for Lzma {
    fn name(&self) -> String {
        "rust-lzma".to_string()
    }
    fn settings(&self) -> Option<String> {
        let extreme = (self.preset & lzma::EXTREME_PRESET) != 0;
        if extreme {
            Some(format!("extreme {}", self.preset & (!lzma::EXTREME_PRESET)))
        } else {
            Some(format!("preset {}", self.preset))
        }
    }
}

impl Compressor for Lzma {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        lzma::compress(data, self.preset).context("lzma compression failed")
    }
}

impl Decompressor for Lzma {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        let mut decompressor =
            lzma::LzmaReader::new_decompressor(src).context("couldn't create lzma decompressor")?;
        decompressor
            .read_exact(dst)
            .context("lzma decompression error")?;
        let mut tmp = [0u8];
        match decompressor.read_exact(&mut tmp) {
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(()),
            _ => Err(anyhow::Error::msg(
                "lzma decompression failed: dst too short",
            )),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut schemes = vec![];
    for i in 0..=9 {
        schemes.push(Lzma {
            preset: i | lzma::EXTREME_PRESET,
        });
    }
    for i in 0..=9 {
        schemes.push(Lzma { preset: i });
    }
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
