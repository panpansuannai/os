use crate::sbi::SET_TIMER;

const TICKS_PER_SEC: usize = 100;
const CLOCK_FREQ: usize = 12500000;

pub fn get_time() -> usize {
    riscv::register::time::read()
}
pub fn set_timecmp(timecmp: usize) {
    crate::sbi::sbi_call(SET_TIMER, [timecmp, 0, 0]);
}
pub fn set_next_trigger() {
    set_timecmp(get_time() + CLOCK_FREQ / TICKS_PER_SEC);

}
