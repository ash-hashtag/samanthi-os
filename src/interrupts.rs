use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use x86_64::{
    instructions::port::Port,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

use crate::{
    gdt, hlt_loop, print, println,
    vga_buffer::{console_backspace, WRITER},
};
use pic8259::ChainedPics;
use spin::Mutex;

extern crate alloc;

use alloc::{collections::BTreeMap, string::ToString, vec::Vec};
use alloc::{collections::LinkedList, string::String};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

        idt.page_fault.set_handler_fn(page_fault_handler);

        idt
    };
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore
        ));
    static ref CONSOLE_HISTORY: Mutex<LinkedList<String>> =
        Mutex::new(LinkedList::from([String::new()]));
    static ref MEMORY_FS: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());
}
pub fn init_idt() {
    IDT.load();
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // println!("TIMER INTERRUPTION\n{:#?}", stack_frame);
    // print!(".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT, ERROR CODE: {}\n{:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let mut keyboard = KEYBOARD.lock();
    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(c) => {
                    match c {
                        '\n' => {
                            print!("\n");
                            let mut history = CONSOLE_HISTORY.lock();
                            if let Some(cmd) = history.front() {
                                if (!cmd.is_empty()) {
                                    match cmd.as_str() {
                                        "clear" => WRITER.lock().clear_everything(),
                                        "ls" => {
                                            for key in MEMORY_FS.lock().keys() {
                                                println!("{}", key);
                                            }
                                        }
                                        s if cmd.starts_with("touch ") => {
                                            if let Some((filename, content)) =
                                                &s["touch ".len()..].split_once(' ')
                                            {
                                                MEMORY_FS.lock().insert(
                                                    filename.to_string(),
                                                    content.to_string(),
                                                );
                                                println!("File created {}", filename);
                                            } else {
                                                println!("Usage: ");
                                                println!("  touch <filename> <content>");
                                            }
                                        }
                                        s if cmd.starts_with("cat ") => {
                                            let filename = &s["cat ".len()..];
                                            if filename.is_empty() {
                                                println!("cat: filename can't be empty, usage: ");
                                                println!("cat <filename>");
                                            } else {
                                                if let Some(content) =
                                                    MEMORY_FS.lock().get(filename)
                                                {
                                                    println!("{}", content);
                                                } else {
                                                    println!(
                                                        "cat: No File found with name {}",
                                                        filename
                                                    );
                                                }
                                            }
                                        }
                                        _ => {
                                            println!("unknown command: {}", cmd)
                                        }
                                    }
                                    history.push_front(String::new());
                                    if history.len() > 10 {
                                        history.pop_back();
                                    }
                                }
                            }
                        }
                        c if c as u8 == 8 => {
                            console_backspace();
                            if let Some(cmd) = CONSOLE_HISTORY.lock().front_mut() {
                                cmd.pop();
                            }
                        }
                        '\t' => {
                            print!("  ");
                        }
                        _ => {
                            print!("{}", c);
                            if let Some(cmd) = CONSOLE_HISTORY.lock().front_mut() {
                                cmd.push(c);
                            }
                        }
                    };
                }
                DecodedKey::RawKey(key) => print!("{:?}", key),
            };
        }
    }
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);

    hlt_loop();
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
