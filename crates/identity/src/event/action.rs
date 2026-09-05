use anchor_codec::{DecodeError, DecodeValue, EncodeError, EncodeValue, require_array_length};
use minicbor::Encoder;

use crate::{DecodeIdentityError, DeviceId, KeySet, NextKeyCommitment, PublicKey};

pub(crate) const ROTATE_CONTROL_ACTION_TAG: u16 = 0;
pub(crate) const AUTHORIZE_DEVICE_ACTION_TAG: u16 = 1;
pub(crate) const REVOKE_DEVICE_ACTION_TAG: u16 = 2;
pub(crate) const DEACTIVATE_ACTION_TAG: u16 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityAction {
    RotateControl(RotateControl),
    AuthorizeDevice(AuthorizeDevice),
    RevokeDevice(RevokeDevice),
    Deactivate,
}

impl IdentityAction {
    pub const fn rotate_control(rotation: RotateControl) -> Self {
        Self::RotateControl(rotation)
    }

    pub const fn authorize_device(authorization: AuthorizeDevice) -> Self {
        Self::AuthorizeDevice(authorization)
    }

    pub const fn revoke_device(revocation: RevokeDevice) -> Self {
        Self::RevokeDevice(revocation)
    }

    pub const fn deactivate() -> Self {
        Self::Deactivate
    }
}

impl EncodeValue for IdentityAction {
    fn encode_value(&self, encoder: &mut Encoder<Vec<u8>>) -> Result<(), EncodeError> {
        encoder.array(2)?;

        match self {
            Self::RotateControl(rotation) => {
                encoder.u16(ROTATE_CONTROL_ACTION_TAG)?;
                rotation.encode_value(encoder)?;
            }
            Self::AuthorizeDevice(key) => {
                encoder.u16(AUTHORIZE_DEVICE_ACTION_TAG)?;
                key.encode_value(encoder)?;
            }
            Self::RevokeDevice(device) => {
                encoder.u16(REVOKE_DEVICE_ACTION_TAG)?;
                device.encode_value(encoder)?;
            }
            Self::Deactivate => {
                encoder.u16(DEACTIVATE_ACTION_TAG)?;
                encoder.array(0)?;
            }
        }

        Ok(())
    }
}

impl DecodeValue for IdentityAction {
    type Error = DecodeIdentityError;

    fn decode_value(decoder: &mut minicbor::Decoder<'_>) -> Result<Self, Self::Error> {
        require_array_length(decoder, 2)?;

        let tag = decoder.u16().map_err(DecodeError::from)?;

        match tag {
            ROTATE_CONTROL_ACTION_TAG => {
                let rotation = RotateControl::decode_value(decoder)?;

                Ok(Self::RotateControl(rotation))
            }
            AUTHORIZE_DEVICE_ACTION_TAG => {
                let authorization = AuthorizeDevice::decode_value(decoder)?;

                Ok(Self::AuthorizeDevice(authorization))
            }
            REVOKE_DEVICE_ACTION_TAG => {
                let revocation = RevokeDevice::decode_value(decoder)?;

                Ok(Self::RevokeDevice(revocation))
            }
            DEACTIVATE_ACTION_TAG => {
                require_array_length(decoder, 0)?;

                Ok(Self::Deactivate)
            }

            actual => Err(DecodeIdentityError::Decode(DecodeError::UnsupportedTag {
                actual,
            })),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotateControl {
    control: KeySet,
    commitment: NextKeyCommitment,
}

impl RotateControl {
    pub const fn new(control: KeySet, commitment: NextKeyCommitment) -> Self {
        Self {
            control,
            commitment,
        }
    }

    pub const fn control(&self) -> &KeySet {
        &self.control
    }

    pub const fn commitment(&self) -> &NextKeyCommitment {
        &self.commitment
    }
}

impl EncodeValue for RotateControl {
    fn encode_value(&self, encoder: &mut Encoder<Vec<u8>>) -> Result<(), EncodeError> {
        encoder.array(2)?;
        self.control().encode_value(encoder)?;
        self.commitment().encode_value(encoder)?;

        Ok(())
    }
}

impl DecodeValue for RotateControl {
    type Error = DecodeIdentityError;

    fn decode_value(decoder: &mut minicbor::Decoder<'_>) -> Result<Self, Self::Error> {
        require_array_length(decoder, 2)?;

        let control = KeySet::decode_value(decoder)?;
        let commitment = NextKeyCommitment::decode_value(decoder)?;

        Ok(Self::new(control, commitment))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthorizeDevice {
    key: PublicKey,
}

impl AuthorizeDevice {
    pub const fn new(key: PublicKey) -> Self {
        Self { key }
    }

    pub const fn key(&self) -> &PublicKey {
        &self.key
    }
}

impl EncodeValue for AuthorizeDevice {
    fn encode_value(&self, encoder: &mut Encoder<Vec<u8>>) -> Result<(), EncodeError> {
        encoder.array(1)?;
        self.key().encode_value(encoder)?;

        Ok(())
    }
}

impl DecodeValue for AuthorizeDevice {
    type Error = DecodeIdentityError;

    fn decode_value(decoder: &mut minicbor::Decoder<'_>) -> Result<Self, Self::Error> {
        require_array_length(decoder, 1)?;

        let key = PublicKey::decode_value(decoder)?;

        Ok(Self::new(key))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevokeDevice {
    device: DeviceId,
}

impl RevokeDevice {
    pub const fn new(device: DeviceId) -> Self {
        Self { device }
    }

    pub const fn device(&self) -> &DeviceId {
        &self.device
    }
}

impl EncodeValue for RevokeDevice {
    fn encode_value(&self, encoder: &mut Encoder<Vec<u8>>) -> Result<(), EncodeError> {
        encoder.array(1)?;
        self.device().encode_value(encoder)?;

        Ok(())
    }
}

impl DecodeValue for RevokeDevice {
    type Error = DecodeIdentityError;

    fn decode_value(decoder: &mut minicbor::Decoder<'_>) -> Result<Self, Self::Error> {
        require_array_length(decoder, 1)?;

        let device = DeviceId::decode_value(decoder)?;

        Ok(Self::new(device))
    }
}

#[cfg(test)]
mod tests {
    use anchor_codec::decode;
    use anyhow::Result;

    use super::*;

    #[test]
    fn identity_action_decode_rejects_unsupported_tag() -> Result<()> {
        let mut encoder = Encoder::new(Vec::new());
        encoder.array(2)?;
        encoder.u16(99)?;
        encoder.array(0)?;

        let bytes = encoder.into_writer();

        let result = decode::<IdentityAction>(&bytes);

        assert!(matches!(
            result,
            Err(DecodeIdentityError::Decode(DecodeError::UnsupportedTag {
                actual: 99
            }))
        ));

        Ok(())
    }
}
