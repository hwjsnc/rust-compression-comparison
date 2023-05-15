use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};

struct Lzo {}

impl DescribeScheme for Lzo {
    fn name(&self) -> String {
        "lzo1x-1".to_string()
    }
    fn settings(&self) -> Option<String> {
        None
    }
}

impl Compressor for Lzo {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        let mut vec = vec![0u8; lzo1x_1::worst_compress(data.len())];
        let slice = lzo1x_1::compress_to_slice(data, &mut vec);
        let len = slice.len();
        drop(slice);
        vec.resize(len, 0u8);
        Ok(vec)
    }
}

impl Decompressor for Lzo {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        let expected_len = dst.len();
        let slice = lzo1x_1::decompress_to_slice(src, dst).context("lzo decompression error")?;
        let actual_len = slice.len();
        anyhow::ensure!(
            actual_len == expected_len,
            "lzo decompression error: length mismatch"
        );
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let schemes = vec![Lzo {}];
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
