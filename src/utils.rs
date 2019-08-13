use super::cpu::TargetFlag;

pub fn set_flag(flag: TargetFlag, register: u16) -> u16 {

    let mut result_reg = register;

    match flag {
        TargetFlag::ZFlag => result_reg = set_bit(result_reg, 7),
        TargetFlag::NFlag => result_reg = set_bit(result_reg, 6),
        TargetFlag::HFlag => result_reg = set_bit(result_reg, 5),
        TargetFlag::CFlag => result_reg = set_bit(result_reg, 4),
    }

    result_reg
}

pub fn reset_flag(flag: TargetFlag, register: u16) -> u16 {

    let mut result_reg = register;

    match flag {
        TargetFlag::ZFlag => result_reg = reset_bit(result_reg, 7),
        TargetFlag::NFlag => result_reg = reset_bit(result_reg, 6),
        TargetFlag::HFlag => result_reg = reset_bit(result_reg, 5),
        TargetFlag::CFlag => result_reg = reset_bit(result_reg, 4),
    }

    result_reg
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

pub fn check_bit(value: u8, bit: u8) -> bool {
    (value & (1 << bit)) != 0
}