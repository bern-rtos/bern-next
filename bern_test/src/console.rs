use nb::Error::Other;
use crate::serial::{self, Serial};
use crate::println;
#[cfg(feature = "serial")]
use crate::sprintln;


pub fn handle_user_input() -> u8 {
    loop {
        let mut command: &str = "";
        let mut rx_buffer = [0u8; 128];

        #[cfg(feature = "serial")]
            {
                let ser = unsafe { Serial::steal() };
                match ser.readln(&mut rx_buffer) {
                    Ok(len) => {
                        command = match core::str::from_utf8(&rx_buffer[0..len]) {
                            Ok(c) => c,
                            Err(_) => continue,
                        };
                    },
                    Err(e) => match e {
                        Other(serial::Error::BufferOverrun) => println!("Error: Serial RX buffer overflow"),
                        Other(serial::Error::NoDownlink) => println!("Error: No serial downlink provided"),
                        _ => println!("Error: Unknown serial error"),
                    }
                };
            }

        let test_index = match command.parse::<u8>() {
            Ok(i) => i,
            Err(_) => {
                println!("Error: Could not parse test index");
                continue;
            },
        };
        return test_index;
    }
}

// todo: make nicer
/* ansi terminal colors, see: https://github.com/l-tools/ansi-colors/blob/master/src/colors.rs */
#[cfg(feature = "colored")]
# [macro_export]
macro_rules ! term_green {
    ($string:expr) => {
        concat!("\x1B[32m", $string, "\x1B[97m")
    }
}

#[cfg(feature = "colored")]
# [macro_export]
macro_rules ! term_red {
    ($string:expr) => {
        concat!("\x1B[31m", $string, "\x1B[97m")
    }
}


#[cfg(not(feature = "colored"))]
# [macro_export]
macro_rules ! term_green {
    ($string:expr) => {
        $string
    }
}

#[cfg(not(feature = "colored"))]
# [macro_export]
macro_rules ! term_red {
    ($string:expr) => {
        $string
    }
}