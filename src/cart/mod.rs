use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU16, Ordering};

const NO_MBC_BYTES: [u8; 3] = [0x00, 0x08, 0x09];
const MBC1_BYTES: [u8; 3] = [0x01, 0x02, 0x03];
const MBC2_BYTES: [u8; 2] = [0x05, 0x06];
const MBC3_BYTES: [u8; 5] = [0x0F, 0x10, 0x11, 0x12, 0x13];
const MBC5_BYTES: [u8; 6] = [0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E];
const MBC6_BYTES: [u8; 1] = [0x20];
const MBC7_BYTES: [u8; 1] = [0x22];

pub trait GameboyCart {
    fn read(&self, address: u16) -> u8;
    fn write(&self, address: u16, value: u8);
}

// A cart without a memory bank controller.
pub struct SimpleCart {
    rom_banks: Vec<Vec<AtomicU8>>,
    ram_bank: Vec<AtomicU8>,

    has_ram: AtomicBool,
    ram_enabled: AtomicBool,
}

impl SimpleCart {
    pub fn new(data: Vec<u8>) -> SimpleCart {
        let mut banks: Vec<Vec<AtomicU8>> = Vec::with_capacity(2);
        let mut data_idx = 0;

        let has_ram = data[0x149] != 0;

        log::info!("Loader: Generating cart without memory controller");
        while data_idx < data.len() {
            let mut new_bank = Vec::with_capacity(16384);
            for _idx in 0..16384 {
                new_bank.push(AtomicU8::new(data[data_idx]));
                data_idx += 1;
            }
            banks.push(new_bank);
        }

        SimpleCart {
            rom_banks: banks,
            ram_bank: create_atomic_vec(8192),
            
            has_ram: AtomicBool::from(has_ram),
            ram_enabled: AtomicBool::from(false),
        }
    }
}

impl GameboyCart for SimpleCart {
    fn read(&self, address: u16) -> u8 {
        if address <= 0x3FFF {
            self.rom_banks[0][(address) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0x4000 && address <= 0x7FFF {
            self.rom_banks[1][(address - 0x4000) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.has_ram.load(Ordering::Relaxed) && self.ram_enabled.load(Ordering::Relaxed) {
                self.ram_bank[(address - 0xA000) as usize].load(Ordering::Relaxed)
            }
            else {
                0xFF
            }
        }
        else {
            println!("F {:X}", address);
            0xFF
        }
    }

    fn write(&self, address: u16, value: u8) {
        if self.has_ram.load(Ordering::Relaxed) {
            if self.ram_enabled.load(Ordering::Relaxed) {
                self.ram_bank[(address - 0xA000) as usize].store(value, Ordering::Relaxed);
            }
        }
    }
}


// Carts using MBC1.
pub struct MBC1Cart {
    rom_banks: Vec<Vec<AtomicU8>>,
    ram_banks: Vec<Vec<AtomicU8>>,

    selected_rom_bank: AtomicU8,
    selected_ram_bank: AtomicU8,
    ram_banking_mode: AtomicBool,

    has_ram: AtomicBool,
    ram_enabled: AtomicBool,
}

impl MBC1Cart {
    pub fn new(data: Vec<u8>) -> MBC1Cart {
        let mut banks: Vec<Vec<AtomicU8>> = Vec::new();
        let mut data_idx = 0;

        let has_ram = data[0x149] != 0;

        while data_idx < data.len() {
            let mut new_bank = Vec::with_capacity(16384);
            for _idx in 0..16384 {
                new_bank.push(AtomicU8::new(data[data_idx]));
                data_idx += 1;
            }

            banks.push(new_bank);
        }
        
        log::info!("Loader: Generating cart with MBC1 controller");
        MBC1Cart {
            rom_banks: banks,
            ram_banks: Vec::new(),

            selected_rom_bank: AtomicU8::new(1),
            selected_ram_bank: AtomicU8::new(0),
            ram_banking_mode: AtomicBool::from(false),

            has_ram: AtomicBool::from(has_ram),
            ram_enabled: AtomicBool::from(false),
        }
    }
}

impl GameboyCart for MBC1Cart {
    fn read(&self, address: u16) -> u8 {
        if address <= 0x3FFF {
            self.rom_banks[0][address as usize].load(Ordering::Relaxed)
        }
        else if address >= 0x4000 && address <= 0x7FFF {
            self.rom_banks[self.selected_rom_bank.load(Ordering::Relaxed) as usize]
                [(address - 0x4000) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.has_ram.load(Ordering::Relaxed) && self.ram_enabled.load(Ordering::Relaxed) {
                //self.ram_banks[self.selected_ram_bank as usize][(address - 0xA000) as usize]
                0xFF
            }
            else {
                0xFF
            }
        }
        else {
            0xFF
        }
    }

    fn write(&self, address: u16, value: u8) {
        if address <= 0x1FFF {
            self.ram_enabled.store(value == 0x0A, Ordering::Relaxed);
        }
        else if address >= 0x2000 && address <= 0x3FFF {
            // Sets the lower 5 bits of the selected rom bank register.
            self.selected_rom_bank.store((self.selected_rom_bank.load(Ordering::Relaxed) & 0x60) | (value & 0x1F),
                Ordering::Relaxed);

            if self.selected_rom_bank.load(Ordering::Relaxed) == 0 {
                self.selected_rom_bank.store(1, Ordering::Relaxed);
            }
            else if self.selected_rom_bank.load(Ordering::Relaxed) == 0x20 {
                self.selected_rom_bank.store(0x21, Ordering::Relaxed);
            }
            else if self.selected_rom_bank.load(Ordering::Relaxed) == 0x40 {
                self.selected_rom_bank.store(0x41, Ordering::Relaxed);
            }
            else if self.selected_rom_bank.load(Ordering::Relaxed) == 0x60 {
                self.selected_rom_bank.store(0x61, Ordering::Relaxed);
            }
        }
        else if address >= 0x4000 && address <= 0x5FFF {
            // Depending on the bank selection mode, sets the RAM bank, or the 2 upper bits of
            // the ROM bank.
            if self.ram_banking_mode.load(Ordering::Relaxed) {
                self.selected_ram_bank.store(value, Ordering::Relaxed);
            }
            else {
                self.selected_rom_bank.store((self.selected_rom_bank.load(Ordering::Relaxed) & 0x9F) | (value | 0x60), 
                    Ordering::Relaxed);
                if self.selected_rom_bank.load(Ordering::Relaxed) == 0 {
                    self.selected_rom_bank.store(1, Ordering::Relaxed);
                }
                else if self.selected_rom_bank.load(Ordering::Relaxed) == 0x20 {
                    self.selected_rom_bank.store(0x21, Ordering::Relaxed);
                }
                else if self.selected_rom_bank.load(Ordering::Relaxed) == 0x40 {
                    self.selected_rom_bank.store(0x41, Ordering::Relaxed);
                }
                else if self.selected_rom_bank.load(Ordering::Relaxed) == 0x60 {
                    self.selected_rom_bank.store(0x61, Ordering::Relaxed);
                }
            }
        }
        else if address >= 0x6000 && address <= 0x7FFF {
            self.ram_banking_mode.store(value == 0x01, Ordering::Relaxed);
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.has_ram.load(Ordering::Relaxed) && self.ram_enabled.load(Ordering::Relaxed) {
                //self.ram_banks[self.ram_banking_mode as usize][(address - 0xA000) as usize] = value;
            }
        }
    }
}


// Carts using MBC2.
pub struct MBC2Cart {
    rom_banks: Vec<Vec<AtomicU8>>,
    ram_bank: Vec<AtomicU8>,

    selected_rom_bank: AtomicU8,

    ram_enabled: AtomicBool,
}

impl MBC2Cart {
    pub fn new(data: Vec<u8>) -> MBC2Cart {
        let mut banks: Vec<Vec<AtomicU8>> = Vec::new();
        let mut data_idx = 0;

        while data_idx < data.len() {
            let mut new_bank = Vec::with_capacity(16384);
            for _idx in 0..16384 {
                new_bank.push(AtomicU8::new(data[data_idx]));
                data_idx += 1;
            }

            banks.push(new_bank);
        }
        
        log::info!("Loader: Generating cart with MBC2 controller");
        MBC2Cart {
            rom_banks: banks,
            ram_bank: create_atomic_vec(512),

            selected_rom_bank: AtomicU8::new(1),

            ram_enabled: AtomicBool::from(false),
        }
    }
}

impl GameboyCart for MBC2Cart {
    fn read(&self, address: u16) -> u8 {
        if address <= 0x3FFF {
            self.rom_banks[0][address as usize].load(Ordering::Relaxed)
        }
        else if address >= 0x4000 && address <= 0x7FFF {
            self.rom_banks[self.selected_rom_bank.load(Ordering::Relaxed) as usize]
                [(address - 0x4000) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0xA000 && address <= 0xA1FF {
            if self.ram_enabled.load(Ordering::Relaxed) {
                self.ram_bank[(address - 0xA000) as usize].load(Ordering::Relaxed) & 0x0F
            }
            else {
                0x0F
            }
        }
        else {
            0xFF
        }
    }

    fn write(&self, address: u16, value: u8) {
        if address <= 0x1FFF {
            if (address & 0x100) == 0 {
                self.ram_enabled.store(value != 0, Ordering::Relaxed)
            }
        }
        else if address >= 0x2000 && address <= 0x3FFF {
            if (address & 0x100) != 0 {
                self.selected_rom_bank.store(value & 0x0F, Ordering::Relaxed)
            }
        }
        else if address >= 0xA000 && address <= 0xA1FF {
            self.ram_bank[(address - 0xA000) as usize].store(value & 0x0F, Ordering::Relaxed);
        }
    }
}


// Carts using MBC3.
pub struct MBC3Cart {
    rom_banks: Vec<Vec<AtomicU8>>,
    ram_banks: Vec<Vec<AtomicU8>>,
    rtc_registers: Vec<AtomicU8>,

    selected_rom_bank: AtomicU8,
    selected_ram_bank: AtomicU8,
    selected_rtc_reg: AtomicU8,

    rtc_register_access: AtomicBool,

    has_ram: AtomicBool,
    ram_enabled: AtomicBool,
    rtc_enabled: AtomicBool,
}

impl MBC3Cart {
    pub fn new(data: Vec<u8>) -> MBC3Cart {
        let mut banks: Vec<Vec<AtomicU8>> = Vec::new();
        let mut data_idx = 0;

        let has_ram = data[0x149] != 0;

        while data_idx < data.len() {
            let mut new_bank = Vec::with_capacity(16384);
            for _idx in 0..16384 {
                new_bank.push(AtomicU8::new(data[data_idx]));
                data_idx += 1;
            }

            banks.push(new_bank);
        }
        
        log::info!("Loader: Generating cart with MBC3 controller");
        MBC3Cart {
            rom_banks: banks,
            ram_banks: Vec::new(),
            rtc_registers: create_atomic_vec(5),

            selected_rom_bank: AtomicU8::new(1),
            selected_ram_bank: AtomicU8::new(0),
            selected_rtc_reg: AtomicU8::new(0),

            rtc_register_access: AtomicBool::from(false),

            has_ram: AtomicBool::from(has_ram),
            ram_enabled: AtomicBool::from(false),
            rtc_enabled: AtomicBool::from(false),
        }
    }
}

impl GameboyCart for MBC3Cart {
    fn read(&self, address: u16) -> u8 {
        if address <= 0x3FFF {
            self.rom_banks[0][address as usize].load(Ordering::Relaxed)
        }
        else if address >= 0x4000 && address <= 0x7FFF {
            self.rom_banks[self.selected_rom_bank.load(Ordering::Relaxed) as usize]
                [(address - 0x4000) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.rtc_register_access.load(Ordering::Relaxed) {
                self.rtc_registers[self.selected_rtc_reg.load(Ordering::Relaxed) as usize].load(Ordering::Relaxed)
            }
            else if self.has_ram.load(Ordering::Relaxed) && self.ram_enabled.load(Ordering::Relaxed) {
                self.ram_banks[self.selected_ram_bank.load(Ordering::Relaxed) as usize]
                    [(address - 0xA000) as usize].load(Ordering::Relaxed)
            }
            else {
                0xFF
            }
        }
        else {
            0xFF
        }
    }

    fn write(&self, address: u16, value: u8) {
        if address <= 0x1FFF {
            self.ram_enabled.store(value == 0x0A, Ordering::Relaxed);
            self.rtc_enabled.store(value == 0x0A, Ordering::Relaxed);
        }
        else if address >= 0x2000 && address <= 0x3FFF {
            if value == 0 {
                self.selected_rom_bank.store(1, Ordering::Relaxed);
            }
            else {
                self.selected_rom_bank.store(value, Ordering::Relaxed);
            }
        }
        else if address >= 0x4000 && address <= 0x5FFF {
            if value <= 0x03 {
                self.selected_ram_bank.store(value, Ordering::Relaxed);
            }
            else if value >= 0x08 && value <= 0x0C {
                self.rtc_register_access.store(true, Ordering::Relaxed);
                self.selected_rtc_reg.store(value - 0x08, Ordering::Relaxed);
            }
        }
    }
}


pub struct MBC5Cart {
    rom_banks: Vec<Vec<AtomicU8>>,
    ram_banks: Vec<Vec<AtomicU8>>,

    selected_rom_bank: AtomicU16,
    selected_ram_bank: AtomicU8,

    has_ram: AtomicBool,
    ram_enabled: AtomicBool,
}

impl MBC5Cart {
    pub fn new(data: Vec<u8>) -> MBC5Cart {
        let mut banks: Vec<Vec<AtomicU8>> = Vec::new();
        let mut data_idx = 0;

        let has_ram = data[0x149] != 0;

        while data_idx < data.len() {
            let mut new_bank = Vec::with_capacity(16384);
            for _idx in 0..16384 {
                new_bank.push(AtomicU8::new(data[data_idx]));
                data_idx += 1;
            }

            banks.push(new_bank);
        }
        
        log::info!("Loader: Generating cart with MBC3 controller");
        MBC5Cart {
            rom_banks: banks,
            ram_banks: Vec::new(),

            selected_rom_bank: AtomicU16::new(1),
            selected_ram_bank: AtomicU8::new(0),

            has_ram: AtomicBool::from(has_ram),
            ram_enabled: AtomicBool::from(false),
        }
    }
}

impl GameboyCart for MBC5Cart {
    fn read(&self, address: u16) -> u8 {
        if address <= 0x3FFF {
            self.rom_banks[0][address as usize].load(Ordering::Relaxed)
        }
        else if address >= 0x4000 && address <= 0x7FFF {
            self.rom_banks[self.selected_rom_bank.load(Ordering::Relaxed) as usize]
                [(address - 0x4000) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.has_ram.load(Ordering::Relaxed) && self.ram_enabled.load(Ordering::Relaxed) {
                self.ram_banks[self.selected_ram_bank.load(Ordering::Relaxed) as usize]
                    [(address - 0xA000) as usize].load(Ordering::Relaxed)
            }
            else {
                0xFF
            }
        }
        else {
            0xFF
        }
    }

    fn write(&self, address: u16, value: u8) {
        if address <= 0x1FFF {
            self.ram_enabled.store(value == 0x0A, Ordering::Relaxed);
        }
        else if address >= 0x2000 && address <= 0x3FFF {
            let current_bnk = self.selected_rom_bank.load(Ordering::Relaxed);

            if address <= 0x2FFF {
                let value = value as u16;
                self.selected_rom_bank.store((current_bnk & 0x100) | value, Ordering::Relaxed);
            }
            else {
                let value = (value as u16) << 8;
                self.selected_rom_bank.store((current_bnk & 0xFF) | value, Ordering::Relaxed);
            }
        }
        else if address >= 0x4000 && address <= 0x5FFF {
            self.selected_ram_bank.store(value, Ordering::Relaxed);
        }
    }
}


pub fn new_cart(data: Vec<u8>) -> Box<dyn GameboyCart + Send + Sync> {
    let cart_type = data[0x147];

    if NO_MBC_BYTES.contains(&cart_type) {
        return Box::from(SimpleCart::new(data))
    }
    if MBC1_BYTES.contains(&cart_type) {
        return Box::from(MBC1Cart::new(data))
    }
    if MBC2_BYTES.contains(&cart_type) {
        return Box::from(MBC2Cart::new(data))
    }
    if MBC3_BYTES.contains(&cart_type) {
        return Box::from(MBC3Cart::new(data))
    }
    if MBC5_BYTES.contains(&cart_type) {
        return Box::from(MBC5Cart::new(data))
    }
    if MBC6_BYTES.contains(&cart_type) {
        unimplemented!();
    }
    if MBC7_BYTES.contains(&cart_type) {
        unimplemented!();
    }
    
    panic!("Unhandled cart type {:X}", cart_type)
}

pub fn dummy_cart() -> Box<dyn GameboyCart + Send + Sync> {
    Box::from(SimpleCart::new(vec![0xFF; 32768]))
}

fn create_atomic_vec(size: usize) -> Vec<AtomicU8> {
    let mut result = Vec::with_capacity(size);

    for _foo in 0..size {
        result.push(AtomicU8::new(0xFF));
    }

    result
}