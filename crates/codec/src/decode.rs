use minicbor::Decoder;

use crate::{CanonicalEncode, DecodeError, EncodeValue, encode, encode_list};

pub trait DecodeValue: Sized {
    type Error: From<DecodeError>;

    fn decode_value(decoder: &mut Decoder<'_>) -> Result<Self, Self::Error>;
}

pub fn decode<T>(bytes: &[u8]) -> Result<T, T::Error>
where
    T: DecodeValue + CanonicalEncode,
{
    let mut decoder = Decoder::new(bytes);
    let value = T::decode_value(&mut decoder)?;

    if decoder.position() != bytes.len() {
        return Err(DecodeError::TrailingBytes.into());
    }

    if encode(&value).map_err(DecodeError::from)? != bytes {
        return Err(DecodeError::Noncanonical.into());
    }

    Ok(value)
}

pub fn decode_array<T>(decoder: &mut Decoder<'_>, maximum: usize) -> Result<Vec<T>, T::Error>
where
    T: DecodeValue,
{
    let count = read_bounded_array_length(decoder, maximum)?;
    let mut items = Vec::with_capacity(count as usize);

    for _ in 0..count {
        items.push(T::decode_value(decoder)?);
    }

    Ok(items)
}

pub fn decode_list<T>(bytes: &[u8], maximum: usize) -> Result<Vec<T>, T::Error>
where
    T: DecodeValue + EncodeValue,
{
    let mut decoder = Decoder::new(bytes);
    let items = decode_array::<T>(&mut decoder, maximum)?;

    if decoder.position() != bytes.len() {
        return Err(DecodeError::TrailingBytes.into());
    }

    if encode_list(&items).map_err(DecodeError::from)? != bytes {
        return Err(DecodeError::Noncanonical.into());
    }

    Ok(items)
}

pub fn require_array_length(decoder: &mut Decoder<'_>, expected: u64) -> Result<(), DecodeError> {
    let actual = decoder.array()?.ok_or(DecodeError::IndefiniteArray)?;

    if actual != expected {
        return Err(DecodeError::UnexpectedArrayLength { expected, actual });
    }

    Ok(())
}

pub fn read_bounded_array_length(
    decoder: &mut Decoder<'_>,
    maximum: usize,
) -> Result<u64, DecodeError> {
    let actual = decoder.array()?.ok_or(DecodeError::IndefiniteArray)?;

    if actual > maximum as u64 {
        return Err(DecodeError::CollectionTooLarge { maximum, actual });
    }

    Ok(actual)
}

pub fn read_bounded_map_length(
    decoder: &mut Decoder<'_>,
    maximum: usize,
) -> Result<u64, DecodeError> {
    let actual = decoder.map()?.ok_or(DecodeError::IndefiniteMap)?;

    if actual > maximum as u64 {
        return Err(DecodeError::CollectionTooLarge { maximum, actual });
    }

    Ok(actual)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use minicbor::Encoder;

    use crate::{EncodeError, FixedBytesArray, encode_array};

    use super::*;

    type Id = FixedBytesArray<4>;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Ids(Vec<Id>);

    impl EncodeValue for Ids {
        fn encode_value(&self, encoder: &mut Encoder<Vec<u8>>) -> Result<(), EncodeError> {
            encode_array(encoder, &self.0)
        }
    }

    impl DecodeValue for Ids {
        type Error = DecodeError;

        fn decode_value(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
            Ok(Self(decode_array(decoder, 8)?))
        }
    }

    #[test]
    fn array_round_trips() -> Result<()> {
        let value = Ids(vec![
            Id::from_bytes([1, 2, 3, 4]),
            Id::from_bytes([5, 6, 7, 8]),
        ]);
        let bytes = encode(&value)?;

        assert_eq!(decode::<Ids>(&bytes)?, value);

        Ok(())
    }

    #[test]
    fn list_round_trips() -> Result<()> {
        let value = vec![Id::from_bytes([1, 2, 3, 4]), Id::from_bytes([5, 6, 7, 8])];
        let bytes = encode_list(&value)?;

        assert_eq!(decode_list::<Id>(&bytes, 8)?, value);

        Ok(())
    }

    #[test]
    fn require_array_length_accepts_matching_length() -> Result<()> {
        let mut encoder = Encoder::new(Vec::new());
        encoder.array(3)?;

        let bytes = encoder.into_writer();
        let mut decoder = Decoder::new(&bytes);

        require_array_length(&mut decoder, 3)?;

        Ok(())
    }

    #[test]
    fn require_array_length_rejects_mismatched_length() -> Result<()> {
        let mut encoder = Encoder::new(Vec::new());
        encoder.array(2)?;

        let bytes = encoder.into_writer();
        let mut decoder = Decoder::new(&bytes);

        let err = require_array_length(&mut decoder, 3).unwrap_err();

        assert!(matches!(
            err,
            DecodeError::UnexpectedArrayLength {
                expected: 3,
                actual: 2
            }
        ));

        Ok(())
    }

    #[test]
    fn require_array_length_rejects_indefinite_array() -> Result<()> {
        let mut encoder = Encoder::new(Vec::new());
        encoder.begin_array()?.end()?;

        let bytes = encoder.into_writer();
        let mut decoder = Decoder::new(&bytes);

        let err = require_array_length(&mut decoder, 0).unwrap_err();

        assert!(matches!(err, DecodeError::IndefiniteArray));

        Ok(())
    }

    #[test]
    fn read_bounded_array_length_returns_actual_length() -> Result<()> {
        let mut encoder = Encoder::new(Vec::new());
        encoder.array(2)?;

        let bytes = encoder.into_writer();
        let mut decoder = Decoder::new(&bytes);

        assert_eq!(read_bounded_array_length(&mut decoder, 5)?, 2);

        Ok(())
    }

    #[test]
    fn read_bounded_array_length_rejects_over_maximum() -> Result<()> {
        let mut encoder = Encoder::new(Vec::new());
        encoder.array(6)?;

        let bytes = encoder.into_writer();
        let mut decoder = Decoder::new(&bytes);

        let err = read_bounded_array_length(&mut decoder, 5).unwrap_err();

        assert!(matches!(
            err,
            DecodeError::CollectionTooLarge {
                maximum: 5,
                actual: 6
            }
        ));

        Ok(())
    }

    #[test]
    fn read_bounded_array_length_rejects_indefinite_array() -> Result<()> {
        let mut encoder = Encoder::new(Vec::new());
        encoder.begin_array()?.end()?;

        let bytes = encoder.into_writer();
        let mut decoder = Decoder::new(&bytes);

        let err = read_bounded_array_length(&mut decoder, 5).unwrap_err();

        assert!(matches!(err, DecodeError::IndefiniteArray));

        Ok(())
    }

    #[test]
    fn read_bounded_map_length_returns_actual_length() -> Result<()> {
        let mut encoder = Encoder::new(Vec::new());
        encoder.map(2)?;

        let bytes = encoder.into_writer();
        let mut decoder = Decoder::new(&bytes);

        assert_eq!(read_bounded_map_length(&mut decoder, 5)?, 2);

        Ok(())
    }

    #[test]
    fn read_bounded_map_length_rejects_over_maximum() -> Result<()> {
        let mut encoder = Encoder::new(Vec::new());
        encoder.map(6)?;

        let bytes = encoder.into_writer();
        let mut decoder = Decoder::new(&bytes);

        let err = read_bounded_map_length(&mut decoder, 5).unwrap_err();

        assert!(matches!(
            err,
            DecodeError::CollectionTooLarge {
                maximum: 5,
                actual: 6
            }
        ));

        Ok(())
    }

    #[test]
    fn read_bounded_map_length_rejects_indefinite_map() -> Result<()> {
        let mut encoder = Encoder::new(Vec::new());
        encoder.begin_map()?.end()?;

        let bytes = encoder.into_writer();
        let mut decoder = Decoder::new(&bytes);

        let err = read_bounded_map_length(&mut decoder, 5).unwrap_err();

        assert!(matches!(err, DecodeError::IndefiniteMap));

        Ok(())
    }
}
