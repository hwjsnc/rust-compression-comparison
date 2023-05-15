use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};

struct Snappy {}

impl DescribeScheme for Snappy {
    fn name(&self) -> String {
        "xsnappy".to_string()
    }
    fn settings(&self) -> Option<String> {
        None
    }
}

impl Compressor for Snappy {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        let mut vec = vec![0u8; xsnappy::max_encode_len(data.len())];
        let len = xsnappy::encode(&mut vec, data);
        vec.resize(len, 0u8);
        Ok(vec)
    }
}

impl Decompressor for Snappy {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        let len = xsnappy::decode(dst, src)
            .map_err(anyhow::Error::msg)
            .context("snappy decode error")?;
        anyhow::ensure!(len == dst.len(), "snappy decode error: length mismatch");
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let schemes = [Snappy {}];
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
