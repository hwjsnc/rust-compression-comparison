use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};

struct LzssDyn(lzss::LzssDyn);

struct LzssStatic<const EI: usize, const EJ: usize, const C: u8, const N: usize, const N2: usize>();

impl DescribeScheme for LzssDyn {
    fn name(&self) -> String {
        "lzss (dyn)".to_string()
    }
    fn settings(&self) -> Option<String> {
        let LzssDyn(compressor) = self;
        Some(format!("ei={}/ej={}", compressor.ei(), compressor.ej()))
    }
}

impl<const EI: usize, const EJ: usize, const C: u8, const N: usize, const N2: usize> DescribeScheme
    for LzssStatic<EI, EJ, C, N, N2>
{
    fn name(&self) -> String {
        "lzss (static)".to_string()
    }
    fn settings(&self) -> Option<String> {
        Some(format!("ei={}/ej={}", EI, EJ))
    }
}

impl Compressor for LzssDyn {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        let Self(compressor) = self;
        compressor
            .compress(
                lzss::SliceReader::new(data),
                lzss::VecWriter::with_capacity(0),
            )
            .context("LzssDyn compression error")
    }
}

impl<const EI: usize, const EJ: usize, const C: u8, const N: usize, const N2: usize> Compressor
    for LzssStatic<EI, EJ, C, N, N2>
{
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        lzss::Lzss::<EI, EJ, C, N, N2>::compress_stack(
            lzss::SliceReader::new(data),
            lzss::VecWriter::with_capacity(0),
        )
        .context("LzssDyn compression error")
    }
}

impl Decompressor for LzssDyn {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        let Self(compressor) = self;
        compressor
            .decompress(
                lzss::SliceReader::new(src),
                lzss::SliceWriterExact::new(dst),
            )
            .context("LzssDyn compression error")
    }
}

impl<const EI: usize, const EJ: usize, const C: u8, const N: usize, const N2: usize> Decompressor
    for LzssStatic<EI, EJ, C, N, N2>
{
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        lzss::Lzss::<EI, EJ, C, N, N2>::decompress_stack(
            lzss::SliceReader::new(src),
            lzss::SliceWriterExact::new(dst),
        )
        .context("LzssDyn compression error")
    }
}

trait Foo: Compressor + Decompressor + DescribeScheme {}
impl Foo for LzssDyn {}
impl<const EI: usize, const EJ: usize, const C: u8, const N: usize, const N2: usize> Foo
    for LzssStatic<EI, EJ, C, N, N2>
{
}

fn main() -> anyhow::Result<()> {
    let mut schemes: std::vec::Vec<Box<dyn Foo>> = vec![];

    schemes.push(Box::new(
        LzssStatic::<10, 4, 0x20, { 1 << 10 }, { 2 << 10 }>(),
    ));
    schemes.push(Box::new(
        LzssStatic::<11, 4, 0x20, { 1 << 11 }, { 2 << 11 }>(),
    ));
    schemes.push(Box::new(
        LzssStatic::<12, 4, 0x20, { 1 << 12 }, { 2 << 12 }>(),
    ));
    schemes.push(Box::new(
        LzssStatic::<13, 4, 0x20, { 1 << 13 }, { 2 << 13 }>(),
    ));
    schemes.push(Box::new(
        LzssStatic::<10, 5, 0x20, { 1 << 10 }, { 2 << 10 }>(),
    ));
    schemes.push(Box::new(
        LzssStatic::<11, 5, 0x20, { 1 << 11 }, { 2 << 11 }>(),
    ));
    schemes.push(Box::new(
        LzssStatic::<12, 5, 0x20, { 1 << 12 }, { 2 << 12 }>(),
    ));
    schemes.push(Box::new(
        LzssStatic::<13, 5, 0x20, { 1 << 13 }, { 2 << 13 }>(),
    ));

    let c = 0x20;
    for ei in 10..=13 {
        for ej in 4..=5 {
            schemes.push(Box::new(LzssDyn(
                lzss::LzssDyn::new(ei, ej, c).context("couldn't create LzssDyn")?,
            )));
        }
    }

    benchmark::<_, _, dyn Foo, _, _>(std::io::stdout(), schemes).context("benchmark failed")
}
