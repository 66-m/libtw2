use packer::UnexpectedEnd;
use packer::Unpacker;
use serde_json;
use std::borrow::Cow;

pub use self::item::Item;

pub mod item;

pub const MAGIC_LEN: usize = 16;
pub const UUID: [u8; MAGIC_LEN] = [
    // "699db17b-8efb-34ff-b1d8-da6f60c15dd1"
    0x69, 0x9d, 0xb1, 0x7b, 0x8e, 0xfb, 0x34, 0xff,
    0xb1, 0xd8, 0xda, 0x6f, 0x60, 0xc1, 0x5d, 0xd1,
];

#[derive(Debug)]
pub struct Header<'a> {
    pub version: i32,
    pub map_name: Cow<'a, str>,
    pub map_size: u32,
    pub map_crc: u32,
}

#[derive(Debug)]
pub enum MaybeEnd<E> {
    Err(E),
    UnexpectedEnd,
}

impl<E> From<UnexpectedEnd> for MaybeEnd<E> {
    fn from(_: UnexpectedEnd) -> MaybeEnd<E> {
        MaybeEnd::UnexpectedEnd
    }
}

#[derive(Debug)]
pub enum HeaderError {
    WrongMagic,
    MalformedJson,
    MalformedHeader,
    MalformedVersion,
    MalformedMapSize,
    MalformedMapCrc,
}

impl From<WrongMagic> for HeaderError {
    fn from(_: WrongMagic) -> HeaderError {
        HeaderError::WrongMagic
    }
}

impl From<HeaderError> for MaybeEnd<HeaderError> {
    fn from(e: HeaderError) -> MaybeEnd<HeaderError> {
        MaybeEnd::Err(e)
    }
}

#[derive(Debug)]
pub struct WrongMagic;

impl From<WrongMagic> for MaybeEnd<WrongMagic> {
    fn from(e: WrongMagic) -> MaybeEnd<WrongMagic> {
        MaybeEnd::Err(e)
    }
}

pub fn read_magic(p: &mut Unpacker) -> Result<(), MaybeEnd<WrongMagic>> {
    let magic = p.read_raw(MAGIC_LEN)?;
    if magic != UUID {
        return Err(WrongMagic.into());
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct JsonHeader<'a> {
    version: Cow<'a, str>,
    map_name: Cow<'a, str>,
    map_size: Cow<'a, str>,
    map_crc: Cow<'a, str>,
}

pub fn read_header<'a>(p: &mut Unpacker<'a>)
    -> Result<Header<'a>, MaybeEnd<HeaderError>>
{
    use self::HeaderError::*;
    let header_data = p.read_string()?;
    let json_header: JsonHeader = serde_json::from_slice(header_data)
        .map_err(|e| if e.is_data() { MalformedHeader } else { MalformedJson })?;
    let header = Header {
        version: json_header.version.parse().map_err(|_| MalformedVersion)?,
        map_name: json_header.map_name,
        map_size: json_header.map_size.parse().map_err(|_| MalformedMapSize)?,
        map_crc: u32::from_str_radix(&json_header.map_crc, 16).map_err(|_| MalformedMapCrc)?,
    };
    Ok(header)
}

impl From<HeaderError> for Error {
    fn from(e: HeaderError) -> Error {
        Error::Header(e)
    }
}

impl From<item::Error> for Error {
    fn from(e: item::Error) -> Error {
        Error::Item(e)
    }
}

#[derive(Debug)]
pub enum Error {
    Header(HeaderError),
    Item(item::Error),
    UnknownVersion,
    TickOverflow,
    UnexpectedEnd,
    InvalidClientId,
    PlayerNewDuplicate,
    PlayerDiffWithoutNew,
    PlayerOldWithoutNew,
    InputNewDuplicate,
    InputDiffWithoutNew,
}

#[cfg(test)]
mod test {
    #[test]
    fn correct_uuid() {
        use super::UUID;
        use uuid::Uuid;

        const UUID_TEEWORLDS: [u8; 16] = [
            // "e05ddaaa-c4e6-4cfb-b642-5d48e80c0029"
            0xe0, 0x5d, 0xda, 0xaa, 0xc4, 0xe6, 0x4c, 0xfb,
            0xb6, 0x42, 0x5d, 0x48, 0xe8, 0x0c, 0x00, 0x29,
        ];
        const UUID_STRING: &'static str = "teehistorian@ddnet.tw";

        let ns = Uuid::from_bytes(&UUID_TEEWORLDS).unwrap();
        let ours = Uuid::from_bytes(&UUID).unwrap();
        let correct = Uuid::new_v3(&ns, UUID_STRING);
        assert_eq!(ours, correct);
    }
}
