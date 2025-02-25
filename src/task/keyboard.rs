extern crate alloc;

use core::task::Poll;

use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};
use conquer_once::spin::OnceCell;
use crossbeam::queue::ArrayQueue;
use futures_util::{task::AtomicWaker, Stream, StreamExt};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, HandleControl, KeyCode, Keyboard, ScancodeSet1};
use spin::Mutex;
use vga::{
    colors::Color16,
    writers::{
        Graphics1280x800x256, Graphics320x200x256, Graphics320x240x256, GraphicsWriter, Text80x25,
        TextWriter,
    },
};

use crate::{
    logging::LOGS,
    print, println, serial_println,
    vga_buffer::{console_backspace, string_to_color, Color, WRITER},
};

use super::executor::EXIT_FLAG;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!(
                "WARNING: scancode queue full; dropping keyboard input {}",
                scancode
            );
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");

        Self { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("SCANCODE_QUEUE not initialized");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());

        match queue.pop() {
            Some(c) => {
                WAKER.take();
                Poll::Ready(Some(c))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();

    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    let mut line = String::new();
    let mut current_dir = String::from("/");

    print!("\n{} $ ", current_dir);
    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    pc_keyboard::DecodedKey::RawKey(key) => {
                        if key == KeyCode::LControl
                            || key == KeyCode::RControl
                            || key == KeyCode::RControl2
                        {
                            print!("^C\n{} $", current_dir);
                            line.clear();
                        } else {
                            serial_println!("unhandled key {:?}", key);
                        }
                    }
                    pc_keyboard::DecodedKey::Unicode(c) => {
                        // backspace
                        if c as u8 == 8 {
                            line.pop();
                            console_backspace();
                            continue;
                        } else if c as u8 == 27 {
                            // let text = Text80x25::new();
                            // text.set_mode();
                            // text.clear_screen();
                            WRITER.lock().clear_everything();
                            print!("{} $ ", current_dir);
                            continue;
                        }

                        print!("{}", c);
                        if c == '\n' && !line.is_empty() {
                            execute_cmd(&mut current_dir, line.as_str());
                            line.clear();
                            print!("{} $ ", current_dir);
                        } else {
                            line.push(c);
                        }
                    }
                }
            }
        }
    }
}

pub enum MemoryFile {
    Static(&'static [u8]),
    Dynamic(Vec<u8>),
}

impl AsRef<[u8]> for MemoryFile {
    fn as_ref(&self) -> &[u8] {
        match self {
            MemoryFile::Static(r) => r,
            MemoryFile::Dynamic(r) => r.as_slice(),
        }
    }
}

lazy_static! {
    static ref MEMORY_FS: Mutex<BTreeMap<String, MemoryFile>> = Mutex::new(BTreeMap::new());
}

pub fn init_memory_fs() {
    let mut fs = MEMORY_FS.lock();
    fs.insert(
        "/wallpaper".into(),
        MemoryFile::Static(include_bytes!("../../assets/wallpaper.jpg.vga")),
    );
    fs.insert(
        "/anime".into(),
        MemoryFile::Static(include_bytes!("../../assets/anime.jpg.vga")),
    );
    fs.insert(
        "/anime2".into(),
        MemoryFile::Static(include_bytes!("../../assets/anime2.jpg.vga")),
    );
    fs.insert(
        "/car".into(),
        MemoryFile::Static(include_bytes!("../../assets/car.jpg.vga")),
    );
}

const FS_SEP: char = '/';

pub fn execute_cmd(current_dir: &mut String, cmd: &str) {
    match cmd {
        "clear" => WRITER.lock().clear_everything(),
        "ls" => {
            let fs = MEMORY_FS.lock();

            for (k, v) in fs.range(current_dir.to_string()..) {
                if !k.starts_with(current_dir.as_str()) {
                    break;
                }
                println!("{:8} {}", v.as_ref().len(), k);
            }
        }
        arg if cmd.starts_with("cd ") => match &arg["cd ".len()..] {
            ".." => {
                if let Some(index) = current_dir.rfind(|c| c == FS_SEP) {
                    current_dir.truncate(index.max(1));
                }
            }
            dirname => {
                if !current_dir.ends_with(FS_SEP) {
                    current_dir.push(FS_SEP);
                }
                current_dir.push_str(dirname);
                if current_dir.ends_with(FS_SEP) {
                    current_dir.pop();
                }
            }
        },

        _ if cmd.starts_with("show ") => {
            let fs = MEMORY_FS.lock();
            let mut filepath = String::new();
            join_paths(&current_dir, &cmd["show ".len()..], &mut filepath);
            if let Some(image) = fs.get(&filepath) {
                unsafe {
                    // let graphics = Graphics1280x800x256::new();
                    let graphics = Graphics320x200x256::new();
                    // let graphics = Graphics320x240x256::new();
                    graphics.set_mode();
                    graphics
                        .get_frame_buffer()
                        .copy_from(image.as_ref().as_ptr(), image.as_ref().len());
                };
            } else {
                println!("image {} not found", filepath);
            }
        }

        args if cmd.starts_with("cat ") => {
            let fs = MEMORY_FS.lock();
            let mut filepath = String::new();
            for arg in args["cat ".len()..].split_whitespace() {
                join_paths(&current_dir, arg, &mut filepath);
                if let Some(content) = fs.get(&filepath) {
                    let content = String::from_utf8_lossy(content.as_ref());
                    println!("{content}");
                } else {
                    println!("cat: {} not found", filepath);
                    break;
                }
            }
        }
        args if cmd.starts_with("rm ") => {
            let mut fs = MEMORY_FS.lock();
            let mut filepath = String::new();
            for arg in args["rm ".len()..].split_whitespace() {
                join_paths(&current_dir, arg, &mut filepath);
                if fs.remove(&filepath).is_some() {
                    println!("Removed {}", filepath);
                } else {
                    println!("File {} not found", filepath);
                }
            }
        }
        args if cmd.starts_with("touch ") => {
            if let Some((filename, content)) = args["touch ".len()..].split_once(' ') {
                let mut filepath = String::new();

                join_paths(&current_dir, filename, &mut filepath);
                if MEMORY_FS
                    .lock()
                    .insert(filepath, MemoryFile::Dynamic(content.as_bytes().to_vec()))
                    .is_some()
                {
                    println!("overwritten {}", filename);
                } else {
                    println!("wrote {} bytes to {}", content.len(), filename);
                }
            } else {
                println!("usage: touch <filename> <...content>");
            }
        }

        _ if cmd.starts_with("color ") => {
            let mut iter = cmd["color ".len()..].split_whitespace();
            if let Some(fg) = iter.next() {
                let bg = iter.next().unwrap_or("black");
                if let (Some(fg), Some(bg)) = (string_to_color(fg), string_to_color(bg)) {
                    WRITER.lock().set_colors(fg, bg);
                    return;
                }
            }

            println!("usage: color <foreground_color> <background_color>");
        }

        _ if cmd.starts_with("read ") => {
            if let Ok(port) = cmd["read ".len()..].parse::<u16>() {
                let val: u8 = unsafe { x86_64::instructions::port::Port::new(port).read() };
                println!("{} {}", port, val);
            } else {
                println!("usage: read <port>");
            }
        }

        "shutdown now" => {
            EXIT_FLAG.store(true, core::sync::atomic::Ordering::Relaxed);
        }
        "logs" => {
            println!("{}", LOGS.lock());
        }
        _ => println!("unknown command or misusage: {}", cmd),
    };
}

pub fn join_paths(path: &str, next: &str, out: &mut String) {
    out.clear();
    if !next.starts_with(FS_SEP) {
        out.push_str(path);
        if !path.ends_with(FS_SEP) {
            out.push(FS_SEP);
        }
    }
    out.push_str(next);
    if out.ends_with(FS_SEP) {
        out.pop();
    }
}
