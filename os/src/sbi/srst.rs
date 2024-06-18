#![allow(unused)]

use super::*;

pub enum ResetType {
    Shutdown,
    ColdReboot,
    WarmReboot,
}

impl ResetType {
    fn value(&self) -> u32 {
        match *self {
            Self::Shutdown => 0x0,
            Self::ColdReboot => 0x1,
            Self::WarmReboot => 0x2,
        }
    }
}

pub enum ResetReason {
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

pub fn system_reset(reset_type: ResetType, reset_reason: ResetReason) -> SbiRet {
    binary::sbi_call_2(
        Eid::SRST,
        0,
        reset_type.value() as _,
        reset_reason.value() as _,
    )
}
