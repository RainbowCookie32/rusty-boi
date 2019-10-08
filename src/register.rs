use super::utils;

pub struct CpuReg {
    pub value: u16,
}

pub struct Pc {
    pub value: u16,
}

pub struct Cycles {
    pub value: u16,
}

pub trait Register {
    fn increment(&mut self) -> bool;
    fn increment_lb(&mut self) -> bool;
    fn increment_rb(&mut self) -> bool;

    fn decrement(&mut self) -> bool;
    fn decrement_lb(&mut self) -> bool;
    fn decrement_rb(&mut self) -> bool;

    fn get_register(&mut self) -> u16;
    fn get_register_lb(&mut self) -> u8;
    fn get_register_rb(&mut self) -> u8;

    fn set_register(&mut self, value: u16);
    fn set_register_lb(&mut self, value: u8);
    fn set_register_rb(&mut self, value: u8);

    fn add_to_reg(&mut self, value: u16) -> bool;
    fn add_to_lb(&mut self, value: u8) -> bool;
    fn add_to_rb(&mut self, value: u8) -> bool;

    fn sub_from_reg(&mut self, value: u16) -> bool;
    fn sub_from_lb(&mut self, value: u8) -> bool;
    fn sub_from_rb(&mut self, value: u8) -> bool;
}

pub trait PcTrait {

    fn add(&mut self, value: u16);
    fn set(&mut self, value: u16);
    fn get(&mut self) -> u16;
}

pub trait CycleCounter {

    fn add(&mut self, value: u16);
    fn get(&self) -> u16;
    fn set(&mut self, value: u16);
}

impl Register for CpuReg {

    fn increment(&mut self) -> bool {
        let result = self.value.overflowing_add(1);
        self.value = result.0;
        result.1
    }
    fn increment_lb(&mut self) -> bool {
        let result = utils::get_lb(self.value).overflowing_add(1);
        self.value = utils::set_lb(self.value, result.0);
        result.1
    }
    fn increment_rb(&mut self) -> bool {
        let result = utils::get_rb(self.value).overflowing_add(1);
        self.value = utils::set_rb(self.value, result.0);
        result.1
    }

    fn decrement(&mut self) -> bool {
        let result = self.value.overflowing_sub(1);
        self.value = result.0;
        result.1
    }
    fn decrement_lb(&mut self) -> bool {
        let result = utils::get_lb(self.value).overflowing_sub(1);
        self.value = utils::set_lb(self.value, result.0);
        result.1
    }
    fn decrement_rb(&mut self) -> bool {
        let result = utils::get_rb(self.value).overflowing_sub(1);
        self.value = utils::set_rb(self.value, result.0);
        result.1
    }

    fn get_register(&mut self) -> u16 {
        self.value
    }
    fn get_register_lb(&mut self) -> u8 {
        utils::get_lb(self.value)
    }
    fn get_register_rb(&mut self) -> u8 {
        utils::get_rb(self.value)
    }

    fn set_register(&mut self, value: u16) {
        self.value = value;
    }
    fn set_register_lb(&mut self, value: u8) {
        self.value = utils::set_lb(self.value, value);
    }
    fn set_register_rb(&mut self, value: u8) {
        self.value = utils::set_rb(self.value, value);
    }

    fn add_to_reg(&mut self, value: u16) -> bool {
        let result = self.value.overflowing_add(value);
        self.value = result.0;
        result.1
    }
    fn add_to_lb(&mut self, value: u8) -> bool {
        let result = utils::get_lb(self.value).overflowing_add(value);
        self.value = utils::set_lb(self.value, result.0);
        result.1
    }
    fn add_to_rb(&mut self, value: u8) -> bool {
        let result = utils::get_rb(self.value).overflowing_add(value);
        self.value = utils::set_rb(self.value, result.0);
        result.1
    }

    fn sub_from_reg(&mut self, value: u16) -> bool {
        let result = self.value.overflowing_sub(value);
        self.value = result.0;
        result.1
    }
    fn sub_from_lb(&mut self, value: u8) -> bool {
        let result = utils::get_lb(self.value).overflowing_sub(value);
        self.value = utils::set_lb(self.value, result.0);
        result.1
    }
    fn sub_from_rb(&mut self, value: u8) -> bool {
        let result = utils::get_rb(self.value).overflowing_sub(value);
        self.value = utils::set_rb(self.value, result.0);
        result.1
    }
}

impl PcTrait for Pc {

    fn add(&mut self, value: u16) {
        self.value += value;
    }

    fn set(&mut self, value: u16) {
        self.value = value;
    }

    fn get(&mut self) -> u16 {
        self.value
    }
}

impl CycleCounter for Cycles {

    fn add(&mut self, value: u16) {
        self.value += value;
    }

    fn get(&self) -> u16 {
        self.value
    }

    fn set(&mut self, value: u16) {
        self.value = value;
    }
}