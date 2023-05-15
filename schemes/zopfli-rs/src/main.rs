use anyhow::Context as _;
use common::{benchmark_compression_only, Compressor, DescribeScheme};

struct Zopfli;

impl DescribeScheme for Zopfli {
    fn name(&self) -> String {
        "zopfli-rs".to_string()
    }
    fn settings(&self) -> Option<String> {
        None
    }
}

impl Compressor for Zopfli {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        let mut vec = vec![];
        zopfli_rs::compress(
            &zopfli_rs::Options {
                verbose: false,
                verbose_more: false,
                iterations: 5,
                block_splitting: true,
                block_splitting_max: 15,
            },
            &zopfli_rs::Format::Deflate,
            data,
            &mut vec,
        )
        .context("zopfli compression error")?;
        Ok(vec)
    }
}

fn main() -> anyhow::Result<()> {
    let schemes = [Zopfli];
    benchmark_compression_only(std::io::stdout(), schemes).context("benchmark failed")
}
