#![no_std]

use bern_conf_type::*;

pub const CONF: Conf = Conf {
    task: Task {
        pool_size: 8,
        priorities: 8,
    },
    event: Event {
        pool_size: 32,
    },
    memory: Memory {
        flash: MemorySection {
            start_address: 0x0800_0000,
            size: Size::S512K,
        },
        sram: MemorySection {
            start_address: 0x2000_0000,
            size: Size::S128K,
        },
        peripheral: MemorySection {
            start_address: 0x4000_0000,
            size: Size::S512M,
        },
    },
};