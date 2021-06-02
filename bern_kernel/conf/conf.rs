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
            start_address: 0x08000000,
            size: Bytes::from_kB(512),
        },
        sram: MemorySection {
            start_address: 0,
            size: Bytes::from_kB(128),
        },
        peripheral: MemorySection {
            start_address: 0x4000_0000,
            size: Bytes::from_MB(512),
        },
    },
};