use std::fs;

fn validate_png_signature(file: &[u8]) {
    const CONTINUE: u8 = 137;

    const P: u8 = 80;
    const N: u8 = 78;
    const G: u8 = 71;

    const CR: u8 = 13;
    const LF: u8 = 10;
    const SUB: u8 = 26;

    assert_eq!(
        &[CONTINUE, P, N, G, CR, LF, SUB, LF],
        &file[..8],
        "file is not a valid png"
    );
}

#[derive(Debug)]
enum ChunkType {
    ImageHeader,
    Palette,
    ImageData,
    ImageTrailer,
    EmbeddedIccProfile,
}

#[derive(Debug)]
enum ColorType {
    GreyScale,
    TrueColor,
    Indexed,
    GreyScaleWithAlpha,
    TrueColorWithAlpha,
}

impl From<u8> for ColorType {
    fn from(value: u8) -> Self {
        match value {
            0 => ColorType::GreyScale,
            2 => ColorType::TrueColor,
            3 => ColorType::Indexed,
            4 => ColorType::GreyScaleWithAlpha,
            6 => ColorType::TrueColorWithAlpha,
            _ => panic!("invalid color_type: {}", value),
        }
    }
}

#[derive(Debug)]
enum CompressionMethod {
    Deflate,
}

impl From<u8> for CompressionMethod {
    fn from(value: u8) -> Self {
        match value {
            0 => CompressionMethod::Deflate,
            _ => panic!("invalid compression method"),
        }
    }
}

#[derive(Debug)]
enum FilterMethod {
    Adaptive,
}

impl From<u8> for FilterMethod {
    fn from(value: u8) -> Self {
        match value {
            0 => FilterMethod::Adaptive,
            _ => panic!("invalid filter method"),
        }
    }
}

#[derive(Debug)]
enum InterlaceMethod {
    None,
    Adam7,
}

impl From<u8> for InterlaceMethod {
    fn from(value: u8) -> Self {
        match value {
            0 => InterlaceMethod::None,
            1 => InterlaceMethod::Adam7,
            _ => panic!("invalid filter method"),
        }
    }
}

#[derive(Debug)]
#[must_use]
struct Chunk<'a> {
    length: u32,
    chunk_type: ChunkType,
    data: &'a [u8],
    checksum: [u8; 4],
    chunk_size: usize,
}

impl<'a> Chunk<'a> {
    fn parse(file: &[u8]) -> Chunk {
        let mut chunk_size = 0;
        let length = u32::from_ne_bytes([file[3], file[2], file[1], file[0]]);
        chunk_size += 4;
        let type_bytes = &file[chunk_size..chunk_size + 4];
        let chunk_type = match type_bytes {
            b"IHDR" => ChunkType::ImageHeader,
            b"PLTE" => ChunkType::Palette,
            b"IDAT" => ChunkType::ImageData,
            b"IEND" => ChunkType::ImageTrailer,
            b"iCCP" => ChunkType::EmbeddedIccProfile,
            _ => panic!("invalid chunk type {type_bytes:?}"),
        };

        chunk_size += 4;

        let data = &file[chunk_size..chunk_size + length as usize];

        chunk_size += length as usize;

        let checksum = [
            file[chunk_size],
            file[chunk_size + 1],
            file[chunk_size + 2],
            file[chunk_size + 3],
        ];

        chunk_size += 4;

        Chunk {
            chunk_type,
            data,
            length,
            chunk_size,
            checksum,
        }
    }
}

#[derive(Debug)]
struct ImageHeader<'a> {
    raw_chunk: &'a Chunk<'a>,
    height: u32,
    width: u32,
    bit_depth: u8,
    color_type: ColorType,
    compression_method: CompressionMethod,
    filter_method: FilterMethod,
    interlace_method: InterlaceMethod,
}

impl<'a> From<&'a Chunk<'a>> for ImageHeader<'a> {
    fn from(raw_chunk: &'a Chunk<'a>) -> Self {
        let width = u32::from_ne_bytes([
            raw_chunk.data[3],
            raw_chunk.data[2],
            raw_chunk.data[1],
            raw_chunk.data[0],
        ]);

        let height = u32::from_ne_bytes([
            raw_chunk.data[7],
            raw_chunk.data[6],
            raw_chunk.data[5],
            raw_chunk.data[4],
        ]);

        let bit_depth = raw_chunk.data[8];
        let color_type = ColorType::from(raw_chunk.data[9]);
        let compression_method = CompressionMethod::from(raw_chunk.data[10]);
        let filter_method = FilterMethod::from(raw_chunk.data[11]);
        let interlace_method = InterlaceMethod::from(raw_chunk.data[12]);

        ImageHeader {
            raw_chunk,
            width,
            height,
            bit_depth,
            color_type,
            compression_method,
            filter_method,
            interlace_method,
        }
    }
}

fn main() {
    let file = fs::read("sample.png").expect("failed to read file");
    let mut cursor = 0;
    validate_png_signature(&file);

    cursor += 8;

    let chunk = Chunk::parse(&file[cursor..]);
    let header = ImageHeader::from(&chunk);

    cursor += chunk.chunk_size;

    dbg!(header);
}
