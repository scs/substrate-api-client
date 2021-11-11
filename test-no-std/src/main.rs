#![feature(start, libc, lang_items)]
#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

// DUT
extern crate substrate_api_client;

// The libc crate allows importing functions from C.
extern crate libc;
use core::panic::PanicInfo;
// A list of C functions that are being imported
extern "C" {
    pub fn printf(format: *const u8, ...) -> i32;
}

#[no_mangle]
// The main function, with its input arguments ignored, and an exit status is returned
pub extern "C" fn main(_nargs: i32, _args: *const *const u8) -> i32 {
    // Print "Hello, World" to stdout using printf
    unsafe {
        printf(b"Hello, World!\n" as *const u8);
    }

    // Exit with a return status of 0.
    0
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[alloc_error_handler]
fn foo(_: core::alloc::Layout) -> ! {
    extern "C" {
        fn abort() -> !;
    }
    unsafe { abort() }
}
