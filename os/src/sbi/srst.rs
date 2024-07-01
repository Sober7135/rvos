#![allow(unused)]

use super::*;
use binary::SbiRet;

pub(crate) enum ResetType {
    SHUTDOWN,
    ColdReboot,
    WarmReboot,
}

impl ResetType {
    fn value(&self) -> u32 {
        match *self {
            Self::SHUTDOWN => 0x0,
            Self::ColdReboot => 0x1,
            Self::WarmReboot => 0x2,
        }
    }
}

pub(crate) enum ResetReason {
    NoReason,
    SystemFailure,
}

impl ResetReason {
    fn value(&self) -> u32 {
        match *self {
            Self::NoReason => 0x0,
            Self::SystemFailure => 0x1,
        }
    }
}

pub(crate) fn system_reset(reset_type: ResetType, reset_reason: ResetReason) -> SbiRet {
    binary::sbi_call_2(
        binary::EID::SRST,
        0,
        reset_type.value() as _,
        reset_reason.value() as _,
    )
}
