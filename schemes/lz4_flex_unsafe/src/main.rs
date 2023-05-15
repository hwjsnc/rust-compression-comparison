use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};

struct Lz4 {}

impl DescribeScheme for Lz4 {
    fn name(&self) -> String {
        "lz4_flex".to_string()
    }
    fn settings(&self) -> Option<String> {
        Some("unsafe".to_string())
    }
}

impl Compressor for Lz4 {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        Ok(lz4_flex::block::compress(data))
    }
}

impl Decompressor for Lz4 {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        let len =
            lz4_flex::block::decompress_into(src, dst).context("lz4_flex decompression error")?;
        anyhow::ensure!(len == dst.len(), "dst buffer length does not match");
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let schemes = [Lz4 {}];
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
