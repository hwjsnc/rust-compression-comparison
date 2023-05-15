use anyhow::Context as _;
use common::{benchmark_compression_only, Compressor, DescribeScheme};

struct Zopfli;

impl DescribeScheme for Zopfli {
    fn name(&self) -> String {
        "zopfli".to_string()
    }
    fn settings(&self) -> Option<String> {
        None
    }
}

impl Compressor for Zopfli {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        let mut vec = vec![];
        zopfli::compress(
            &zopfli::Options {
                iteration_count: 5.try_into().unwrap(),
                maximum_block_splits: 15,
            },
            &zopfli::Format::Deflate,
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
