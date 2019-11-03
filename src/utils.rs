use super::register::CpuReg;
use super::register::Register;


pub fn set_zf(value: bool, af: &mut CpuReg) {

    let reg_value = af.get_register_rb();
    if value {
        af.set_register_rb(set_bit(reg_value, 7));
    }
    else {
        af.set_register_rb(reset_bit(reg_value, 7));
    }
}

pub fn set_nf(value: bool, af: &mut CpuReg) {

    let reg_value = af.get_register_rb();
    if value {
        af.set_register_rb(set_bit(reg_value, 6));
    }
    else {
        af.set_register_rb(reset_bit(reg_value, 6));
    }
}

pub fn set_hf(value: bool, af: &mut CpuReg) {

    let reg_value = af.get_register_rb();
    if value {
        af.set_register_rb(set_bit(reg_value, 5));
    }
    else {
        af.set_register_rb(reset_bit(reg_value, 5));
    }
}

pub fn set_cf(value: bool, af: &mut CpuReg) {

    let reg_value = af.get_register_rb();
    if value {
        af.set_register_rb(set_bit(reg_value, 4));
    }
    else {
        af.set_register_rb(reset_bit(reg_value, 4));
    }
}

pub fn get_zf(af: &mut CpuReg) -> bool {
    check_bit(af.get_register_rb(), 7)
}

pub fn get_nf(af: &mut CpuReg) -> bool {
    check_bit(af.get_register_rb(), 6)
}

pub fn get_hf(af: &mut CpuReg) -> bool {
    check_bit(af.get_register_rb(), 5)
}

pub fn get_cf(af: &mut CpuReg) -> bool {
    check_bit(af.get_register_rb(), 4)
}

pub fn get_carry(af: &mut CpuReg) -> u8 {

    let value: u8;
    if check_bit(af.get_register_rb(), 4) { value = 1; }
    else { value = 0; }
    value
}


// assuming 16 bit values is all we ever deal with
// lb means "left byte", rb means "right byte"

// (left and right is used instead of high and low in order to
// prevent confusion when dealing with different endiannesses)

pub fn get_lb(value: u16) -> u8 {
    (value >> 8) as u8
}

pub fn set_lb(value: u16, lb_val: u8) -> u16 {
    value & 0xFF | (lb_val as u16) << 8
}

pub fn get_rb(value: u16) -> u8 {
    (value & 0xFF) as u8
}

pub fn set_rb(value: u16, rb_val: u8) -> u16 {
    value & !0xFF | rb_val as u16
}

pub fn set_bit(value: u8, offset: u8) -> u8 {
    value | 1 << offset
}

pub fn reset_bit(value: u8, offset: u8) -> u8 {
    value & !(1 << offset)
}

pub fn check_bit(value: u8, bit: u8) -> bool {
    (value & (1 << bit)) != 0
}

pub fn check_hf_add(values: (&u8, &u8)) -> bool {
    ((values.0 & 0xF) + (values.1 & 0xF)) & 0x10 == 0x10
}

pub fn check_half_carry_u16(values: (&u16, &u16)) -> bool {
    ((values.0 & 0xFFF) + (values.1 & 0xFFF)) > 0xFFF
}

pub fn check_half_borrow(values: (u8, u8)) -> bool {
    (values.0 & 0xF).overflowing_sub(values.1 & 0xF).1
}

pub fn swap_nibbles(value: u8) -> u8 {
    (value & 0x0F) << 4 | (value & 0xF0) >> 4
}