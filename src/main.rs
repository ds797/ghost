#![no_std]	// Prevents linking of the standard library
#![no_main]	// Prevents bootstrapping with crt0 (C runtime 0)
#![feature(custom_test_frameworks)]
#![test_runner(ghost::test_runner)]
#![reexport_test_harness_main = "test_main"] // Align with no_main

extern crate alloc;

use core::panic::PanicInfo;
use alloc::boxed::Box;
use alloc::{vec, vec::Vec, rc::Rc};
use bootloader::{entry_point, BootInfo};
use ghost::task::keyboard;
use ghost::{allocator, println};
use ghost::task::{executor::Executor, Task};

// Defines panic functions since we no longer have access to std
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);

	ghost::hlt_loop();
}
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	ghost::test_panic_handler(info);
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
	use ghost::{memory, memory::BootInfoFrameAllocator};
	use x86_64::VirtAddr;

	println!("WELCOME_TO_GHOST{}", "!");

	ghost::init();

	let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
	let mut mapper = unsafe { memory::init(phys_mem_offset) };
	let mut frame_allocator = unsafe {
		BootInfoFrameAllocator::init(&boot_info.memory_map)
	};

	allocator::init_heap(&mut mapper, &mut frame_allocator)
		.expect("heap initialization failed");

	let heap_value = Box::new(41);
	println!("heap_value at {:p}", heap_value);

	// create a dynamically sized vector
	let mut vec = Vec::new();
	for i in 0..500 {
			vec.push(i);
	}
	println!("vec at {:p}", vec.as_slice());

	// create a reference counted vector -> will be freed when count reaches 0
	let reference_counted = Rc::new(vec![1, 2, 3]);
	let cloned_reference = reference_counted.clone();
	println!("current reference count is {}", Rc::strong_count(&cloned_reference));
	core::mem::drop(reference_counted);
	println!("reference count is {} now", Rc::strong_count(&cloned_reference));

	println!("asynchronous testing...");

	let mut executor = Executor::new();
	executor.spawn(Task::new(example_task()));
	executor.spawn(Task::new(keyboard::print_keypresses()));
	executor.run();

	#[cfg(test)]
	test_main();

	println!("It didn't crash!");

	ghost::hlt_loop();
}

async fn async_number() -> u32 {
	42
}

async fn example_task() {
	let number = async_number().await;
	println!("async number: {}", number);
}