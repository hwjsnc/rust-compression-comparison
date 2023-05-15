use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};

struct Snappy {}

impl DescribeScheme for Snappy {
    fn name(&self) -> String {
        "tetsy_snappy".to_string()
    }
    fn settings(&self) -> Option<String> {
        None
    }
}

impl Compressor for Snappy {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        Ok(tetsy_snappy::compress(data))
    }
}

impl Decompressor for Snappy {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        let mut vec = std::vec::Vec::with_capacity(dst.len());
        tetsy_snappy::decompress_into(src, &mut vec).context("snappy decompression error")?;
        anyhow::ensure!(
            vec.len() == dst.len(),
            "snappy decompression error: length mismatch",
        );
        dst.copy_from_slice(&vec);
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let schemes = [Snappy {}];
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
