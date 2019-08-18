use super::register::CpuReg;
use super::register::Register;


pub fn set_zf(value: bool, af: &mut CpuReg) {

    if value {
        set_bit(af.get_register_rb() as u16, 7);
    }
    else {
        reset_bit(af.get_register_rb() as u16, 7);
    }
}

pub fn set_nf(value: bool, af: &mut CpuReg) {

    if value {
        set_bit(af.get_register_rb() as u16, 6);
    }
    else {
        reset_bit(af.get_register_rb() as u16, 6);
    }
}

pub fn set_hf(value: bool, af: &mut CpuReg) {

    if value {
        set_bit(af.get_register_rb() as u16, 5);
    }
    else {
        reset_bit(af.get_register_rb() as u16, 5);
    }
}

pub fn set_cf(value: bool, af: &mut CpuReg) {

    if value {
        set_bit(af.get_register_rb() as u16, 4);
    }
    else {
        reset_bit(af.get_register_rb() as u16, 4);
    }
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

pub fn set_bit(value: u16, offset: u8) -> u16 {
    value | 1 << offset
}

pub fn reset_bit(value: u16, offset: u8) -> u16 {
    value & !(1 << offset)
}

pub fn set_bit_u8(value: u8, offset: u8) -> u8 {
    value | 1 << offset
}

pub fn reset_bit_u8(value: u8, offset: u8) -> u8 {
    value & !(1 << offset)
}

pub fn check_bit(value: u8, bit: u8) -> bool {
    (value & (1 << bit)) != 0
}