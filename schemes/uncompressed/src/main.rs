use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};

struct Uncompressed {}

impl DescribeScheme for Uncompressed {
    fn name(&self) -> String {
        "uncompressed".to_string()
    }
    fn settings(&self) -> Option<String> {
        None
    }
}

impl Compressor for Uncompressed {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        Ok(data.to_vec())
    }
}

impl Decompressor for Uncompressed {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        anyhow::ensure!(
            src.len() == dst.len(),
            "destination buffer length doesn't match"
        );
        dst.copy_from_slice(src);
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let schemes = [Uncompressed {}];
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
