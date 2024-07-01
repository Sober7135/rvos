use super::*;

pub(crate) fn set_timer(stime_value: usize) -> SbiRet {
    sbi_call_1(Eid::TIME, 0, stime_value)
}
