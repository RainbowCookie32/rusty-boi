const NO_MBC_BYTES: [u8; 3] = [0x00, 0x08, 0x09];
const MBC1_BYTES: [u8; 3] = [0x01, 0x02, 0x03];
const MBC2_BYTES: [u8; 2] = [0x05, 0x06];
const MBC3_BYTES: [u8; 5] = [0x0F, 0x10, 0x11, 0x12, 0x13];
const MBC5_BYTES: [u8; 6] = [0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E];
const MBC6_BYTES: [u8; 1] = [0x20];
const MBC7_BYTES: [u8; 1] = [0x22];

pub trait GameboyCart {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

// A cart without a memory bank controller.
pub struct SimpleCart {
    rom_banks: Vec<Vec<u8>>,
    ram_bank: Vec<u8>,

    has_ram: bool,
    ram_enabled: bool,
}

impl SimpleCart {
    pub fn new(data: Vec<u8>) -> SimpleCart {
        let mut banks: Vec<Vec<u8>> = Vec::with_capacity(2);
        let mut data_idx = 0;

        let has_ram = data[0x149] != 0;

        log::info!("Loader: Generating cart without memory controller");
        while data_idx < data.len() {
            let mut new_bank = Vec::with_capacity(16384);
            for _idx in 0..16384 {
                new_bank.push(data[data_idx]);
                data_idx += 1;
            }
            banks.push(new_bank);
        }

        SimpleCart {
            rom_banks: banks,
            ram_bank: vec![0xFF; 8192],
            
            has_ram: has_ram,
            ram_enabled: false,
        }
    }
}

impl GameboyCart for SimpleCart {
    fn read(&self, address: u16) -> u8 {
        if address <= 0x3FFF {
            self.rom_banks[0][(address) as usize]
        }
        else if address >= 0x4000 && address <= 0x7FFF {
            self.rom_banks[1][(address - 0x4000) as usize]
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.has_ram && self.ram_enabled {
                self.ram_bank[(address - 0xA000) as usize]
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

    fn write(&mut self, address: u16, value: u8) {
        if self.has_ram {
            if self.ram_enabled {
                self.ram_bank[(address - 0xA000) as usize] = value;
            }
        }
    }
}


// Carts using MBC1.
pub struct MBC1Cart {
    rom_banks: Vec<Vec<u8>>,
    ram_banks: Vec<Vec<u8>>,

    selected_rom_bank: u8,
    selected_ram_bank: u8,
    ram_banking_mode: bool,

    has_ram: bool,
    ram_enabled: bool,
}

impl MBC1Cart {
    pub fn new(data: Vec<u8>) -> MBC1Cart {
        let mut banks: Vec<Vec<u8>> = Vec::new();
        let mut data_idx = 0;

        let has_ram = data[0x149] != 0;

        while data_idx < data.len() {
            let mut new_bank = Vec::with_capacity(16384);
            for _idx in 0..16384 {
                new_bank.push(data[data_idx]);
                data_idx += 1;
            }
            banks.push(new_bank);
        }
        
        log::info!("Loader: Generating cart with MBC1 controller");
        MBC1Cart {
            rom_banks: banks,
            ram_banks: Vec::new(),

            selected_rom_bank: 1,
            selected_ram_bank: 0,
            ram_banking_mode: false,

            has_ram: has_ram,
            ram_enabled: false,
        }
    }
}

impl GameboyCart for MBC1Cart {
    fn read(&self, address: u16) -> u8 {
        if address <= 0x3FFF {
            self.rom_banks[0][address as usize]
        }
        else if address >= 0x4000 && address <= 0x7FFF {
            self.rom_banks[self.selected_rom_bank as usize][(address - 0x4000) as usize]
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.has_ram && self.ram_enabled {
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

    fn write(&mut self, address: u16, value: u8) {
        if address <= 0x1FFF {
            self.ram_enabled = value == 0x0A;
        }
        else if address >= 0x2000 && address <= 0x3FFF {
            // Sets the lower 5 bits of the selected rom bank register.
            self.selected_rom_bank = (self.selected_rom_bank & 0x60) | (value & 0x1F);
            if self.selected_rom_bank == 0 {
                self.selected_rom_bank = 1;
            }
            else if self.selected_rom_bank == 0x20 {
                self.selected_rom_bank = 0x21;
            }
            else if self.selected_rom_bank == 0x40 {
                self.selected_rom_bank = 0x41;
            }
            else if self.selected_rom_bank == 0x60 {
                self.selected_rom_bank = 0x61;
            }
        }
        else if address >= 0x4000 && address <= 0x5FFF {
            // Depending on the bank selection mode, sets the RAM bank, or the 2 upper bits of
            // the ROM bank.
            if self.ram_banking_mode {
                self.selected_ram_bank = value;
            }
            else {
                self.selected_rom_bank = (self.selected_rom_bank & 0x9F) | (value | 0x60);
                if self.selected_rom_bank == 0 {
                    self.selected_rom_bank = 1;
                }
                else if self.selected_rom_bank == 0x20 {
                    self.selected_rom_bank = 0x21;
                }
                else if self.selected_rom_bank == 0x40 {
                    self.selected_rom_bank = 0x41;
                }
                else if self.selected_rom_bank == 0x60 {
                    self.selected_rom_bank = 0x61;
                }
            }
        }
        else if address >= 0x6000 && address <= 0x7FFF {
            self.ram_banking_mode = value == 0x01;
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.has_ram && self.ram_enabled {
                //self.ram_banks[self.ram_banking_mode as usize][(address - 0xA000) as usize] = value;
            }
        }
    }
}


// Carts using MBC2.
pub struct MBC2Cart {

}

impl MBC2Cart {
    pub fn new(data: Vec<u8>) -> MBC2Cart {
        log::warn!("Loader: Generating cart with MBC2 controller");
        MBC2Cart {

        }
    }
}

impl GameboyCart for MBC2Cart {
    fn read(&self, address: u16) -> u8 {
        0
    }

    fn write(&mut self, address: u16, value: u8) {
        
    }
}


// Carts using MBC3.
pub struct MBC3Cart {
    rom_banks: Vec<Vec<u8>>,
    ram_banks: Vec<Vec<u8>>,
    rtc_registers: Vec<u8>,

    selected_rom_bank: u8,
    selected_ram_bank: u8,
    selected_rtc_reg: u8,

    rtc_register_access: bool,

    has_ram: bool,
    ram_enabled: bool,
    rtc_enabled: bool,
}

impl MBC3Cart {
    pub fn new(data: Vec<u8>) -> MBC3Cart {
        let mut banks: Vec<Vec<u8>> = Vec::new();
        let mut data_idx = 0;

        let has_ram = data[0x149] != 0;

        while data_idx < data.len() {
            let mut new_bank = Vec::with_capacity(16384);
            for _idx in 0..16384 {
                new_bank.push(data[data_idx]);
                data_idx += 1;
            }
            banks.push(new_bank);
        }
        
        log::info!("Loader: Generating cart with MBC3 controller");
        MBC3Cart {
            rom_banks: banks,
            ram_banks: Vec::new(),
            rtc_registers: vec![0xFF; 5],

            selected_rom_bank: 1,
            selected_ram_bank: 0,
            selected_rtc_reg: 0,

            rtc_register_access: false,

            has_ram: has_ram,
            ram_enabled: false,
            rtc_enabled: false,
        }
    }
}

impl GameboyCart for MBC3Cart {
    fn read(&self, address: u16) -> u8 {
        if address <= 0x3FFF {
            self.rom_banks[0][address as usize]
        }
        else if address >= 0x4000 && address <= 0x7FFF {
            self.rom_banks[self.selected_rom_bank as usize][(address - 0x4000) as usize]
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.rtc_register_access {
                self.rtc_registers[self.selected_rtc_reg as usize]
            }
            else if self.has_ram && self.ram_enabled {
                self.ram_banks[self.selected_ram_bank as usize][(address - 0xA000) as usize]
            }
            else {
                0xFF
            }
        }
        else {
            0xFF
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if address <= 0x1FFF {
            self.ram_enabled = value == 0x0A;
            self.rtc_enabled = value == 0x0A;
        }
        else if address >= 0x2000 && address <= 0x3FFF {
            if value == 0 {
                self.selected_rom_bank = 1;
            }
            else {
                self.selected_rom_bank = value;
            }
        }
        else if address >= 0x4000 && address <= 0x5FFF {
            if value <= 0x03 {
                self.selected_ram_bank = value;
            }
            else if value >= 0x08 && value <= 0x0C {
                self.rtc_register_access = true;
                self.selected_rtc_reg = value - 0x08;
            }
        }
    }
}

pub fn new_cart(data: Vec<u8>) -> Box<dyn GameboyCart> {
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
    
    panic!("Unhandled cart type {:X}", cart_type)
}

pub fn dummy_cart() -> Box<dyn GameboyCart> {
    Box::from(SimpleCart::new(vec![0xFF; 16384]))
}