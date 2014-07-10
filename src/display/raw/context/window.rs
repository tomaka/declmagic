extern crate std;

use super::libglfw3;
use libc::{ c_int, c_char };
use std::comm::{ Sender, Receiver, Empty, Disconnected };
use std::mem::transmute;
use std::ptr::null;
use std::rc::Rc;
use std::string::String;
use std::sync::{ Arc, Future, Mutex };
use super::super::WindowEvent;
use sync::one::{ Once, ONCE_INIT };
use threaded_executer::CommandsThread;

pub struct Window {
    handle : *const libglfw3::GLFWwindow,
    commands : CommandsThread,
    eventsSender : Box<Sender<WindowEvent>>,
    eventsReceiver : Receiver<WindowEvent>
}

static mut GLFWInitialized: Once = ONCE_INIT;

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            libglfw3::glfwDestroyWindow(self.handle);
        }
    }
}

extern fn errorCallback(errorCode: c_int, description: *const c_char) {
    fail!("error from glfw: {} (code {})", unsafe { std::c_str::CString::new(description, false).as_str().unwrap() }, errorCode);
}

extern fn keyCallback(window: *const libglfw3::GLFWwindow, key: c_int, scancode: c_int, action: c_int, mods: c_int) {
    let sender : &Sender<WindowEvent> = unsafe { transmute(libglfw3::glfwGetWindowUserPointer(window)) };

    let inputKey = match key {
        libglfw3::KEY_SPACE => ::input::Space,
        libglfw3::KEY_APOSTROPHE => ::input::Apostrophe,
        libglfw3::KEY_COMMA => ::input::Comma,
        libglfw3::KEY_MINUS => ::input::Minus,
        libglfw3::KEY_PERIOD => ::input::Period,
        libglfw3::KEY_SLASH => ::input::Slash,
        libglfw3::KEY_0 => ::input::Key0,
        libglfw3::KEY_1 => ::input::Key1,
        libglfw3::KEY_2 => ::input::Key2,
        libglfw3::KEY_3 => ::input::Key3,
        libglfw3::KEY_4 => ::input::Key4,
        libglfw3::KEY_5 => ::input::Key5,
        libglfw3::KEY_6 => ::input::Key6,
        libglfw3::KEY_7 => ::input::Key7,
        libglfw3::KEY_8 => ::input::Key8,
        libglfw3::KEY_9 => ::input::Key9,
        libglfw3::KEY_SEMICOLON => ::input::Semicolon,
        //libglfw3::KEY_EQUAL => ::input::Equal,
        libglfw3::KEY_A => ::input::A,
        libglfw3::KEY_B => ::input::B,
        libglfw3::KEY_C => ::input::C,
        libglfw3::KEY_D => ::input::D,
        libglfw3::KEY_E => ::input::E,
        libglfw3::KEY_F => ::input::F,
        libglfw3::KEY_G => ::input::G,
        libglfw3::KEY_H => ::input::H,
        libglfw3::KEY_I => ::input::I,
        libglfw3::KEY_J => ::input::J,
        libglfw3::KEY_K => ::input::K,
        libglfw3::KEY_L => ::input::L,
        libglfw3::KEY_M => ::input::M,
        libglfw3::KEY_N => ::input::N,
        libglfw3::KEY_O => ::input::O,
        libglfw3::KEY_P => ::input::P,
        libglfw3::KEY_Q => ::input::Q,
        libglfw3::KEY_R => ::input::R,
        libglfw3::KEY_S => ::input::S,
        libglfw3::KEY_T => ::input::T,
        libglfw3::KEY_U => ::input::U,
        libglfw3::KEY_V => ::input::V,
        libglfw3::KEY_W => ::input::W,
        libglfw3::KEY_X => ::input::X,
        libglfw3::KEY_Y => ::input::Y,
        libglfw3::KEY_Z => ::input::Z,
        //libglfw3::KEY_LEFT_BRACKET => ::input::LeftBracket,
        libglfw3::KEY_BACKSLASH => ::input::Backslash,
        //libglfw3::KEY_RIGHT_BRACKET => ::input::RightBracket,
        libglfw3::KEY_GRAVE_ACCENT => ::input::Grave,
        //libglfw3::KEY_WORLD_1 => ::input::World1,
        //libglfw3::KEY_WORLD_2 => ::input::World2,
        libglfw3::KEY_ESCAPE => ::input::Escape,
        libglfw3::KEY_ENTER => ::input::Return,
        libglfw3::KEY_TAB => ::input::Tab,
        libglfw3::KEY_BACKSPACE => ::input::Back,
        libglfw3::KEY_INSERT => ::input::Insert,
        libglfw3::KEY_DELETE => ::input::Delete,
        libglfw3::KEY_RIGHT => ::input::Right,
        libglfw3::KEY_LEFT => ::input::Left,
        libglfw3::KEY_DOWN => ::input::Down,
        libglfw3::KEY_UP => ::input::Up,
        /*libglfw3::KEY_PAGE_UP => ::input::PageUp,
        libglfw3::KEY_PAGE_DOWN => ::input::PageDown,*/
        libglfw3::KEY_HOME => ::input::Home,
        libglfw3::KEY_END => ::input::End,
        //libglfw3::KEY_CAPS_LOCK => ::input::CapsLock,
        //libglfw3::KEY_SCROLL_LOCK => ::input::ScrollLock,
        //libglfw3::KEY_NUM_LOCK => ::input::NumLock,
        //libglfw3::KEY_PRINT_SCREEN => ::input::PrintScreen,
        libglfw3::KEY_PAUSE => ::input::Pause,
        libglfw3::KEY_F1 => ::input::F1,
        libglfw3::KEY_F2 => ::input::F2,
        libglfw3::KEY_F3 => ::input::F3,
        libglfw3::KEY_F4 => ::input::F4,
        libglfw3::KEY_F5 => ::input::F5,
        libglfw3::KEY_F6 => ::input::F6,
        libglfw3::KEY_F7 => ::input::F7,
        libglfw3::KEY_F8 => ::input::F8,
        libglfw3::KEY_F9 => ::input::F9,
        libglfw3::KEY_F10 => ::input::F10,
        libglfw3::KEY_F11 => ::input::F11,
        libglfw3::KEY_F12 => ::input::F12,
        libglfw3::KEY_F13 => ::input::F13,
        libglfw3::KEY_F14 => ::input::F14,
        libglfw3::KEY_F15 => ::input::F15,
        /*libglfw3::KEY_F16 => ::input::F16,
        libglfw3::KEY_F17 => ::input::F17,
        libglfw3::KEY_F18 => ::input::F18,
        libglfw3::KEY_F19 => ::input::F19,
        libglfw3::KEY_F20 => ::input::F20,
        libglfw3::KEY_F21 => ::input::F21,
        libglfw3::KEY_F22 => ::input::F22,
        libglfw3::KEY_F23 => ::input::F23,
        libglfw3::KEY_F24 => ::input::F24,
        libglfw3::KEY_F25 => ::input::F25,*/
        libglfw3::KEY_KP_0 => ::input::Numpad0,
        libglfw3::KEY_KP_1 => ::input::Numpad1,
        libglfw3::KEY_KP_2 => ::input::Numpad2,
        libglfw3::KEY_KP_3 => ::input::Numpad3,
        libglfw3::KEY_KP_4 => ::input::Numpad4,
        libglfw3::KEY_KP_5 => ::input::Numpad5,
        libglfw3::KEY_KP_6 => ::input::Numpad6,
        libglfw3::KEY_KP_7 => ::input::Numpad7,
        libglfw3::KEY_KP_8 => ::input::Numpad8,
        libglfw3::KEY_KP_9 => ::input::Numpad9,
        /*libglfw3::KEY_KP_DECIMAL => ::input::NumpadDecimal,
        libglfw3::KEY_KP_DIVIDE => ::input::NumpadDivide,
        libglfw3::KEY_KP_MULTIPLY => ::input::NumpadMultiply,
        libglfw3::KEY_KP_SUBTRACT => ::input::NumpadSubtract,
        libglfw3::KEY_KP_ADD => ::input::NumpadAdd,*/
        libglfw3::KEY_KP_ENTER => ::input::NumpadEnter,
        //libglfw3::KEY_KP_EQUAL => ::input::NumpadEqual,
        libglfw3::KEY_LEFT_SHIFT => ::input::LShift,
        libglfw3::KEY_LEFT_CONTROL => ::input::LControl,
        //libglfw3::KEY_LEFT_ALT => ::input::LAlt,
        //libglfw3::KEY_LEFT_SUPER => ::input::LeftSuper,
        libglfw3::KEY_RIGHT_SHIFT => ::input::RShift,
        libglfw3::KEY_RIGHT_CONTROL => ::input::RControl,
        //libglfw3::KEY_RIGHT_ALT => ::input::AltGr,
        //libglfw3::KEY_RIGHT_SUPER => ::input::RSuper,
        //libglfw3::KEY_MENU => ::input::Menu,
        _ => return
    };

    sender.send(super::super::Input(match action {
        libglfw3::PRESS => ::input::Pressed(inputKey),
        libglfw3::RELEASE => ::input::Released(inputKey),
        //GLFW_REPEAT => ,
        _ => return
    }));
}

extern fn posCallback(window: *const libglfw3::GLFWwindow, x: c_int, y: c_int) {
    let sender : &Sender<WindowEvent> = unsafe { transmute(libglfw3::glfwGetWindowUserPointer(window)) };
    sender.send(super::super::Moved(x as uint, y as uint));
}

extern fn sizeCallback(window: *const libglfw3::GLFWwindow, w: c_int, h: c_int) {
    let sender : &Sender<WindowEvent> = unsafe { transmute(libglfw3::glfwGetWindowUserPointer(window)) };
    sender.send(super::super::Resized(w as uint, h as uint));
}

extern fn closeCallback(window: *const libglfw3::GLFWwindow) {
    let sender : &Sender<WindowEvent> = unsafe { transmute(libglfw3::glfwGetWindowUserPointer(window)) };
    sender.send(super::super::Closed);
}

extern fn cursorPosCallback(window: *const libglfw3::GLFWwindow, x: ::libc::c_double, y: ::libc::c_double) {
    let mut width: c_int = unsafe { ::std::mem::uninitialized() };
    let mut height: c_int = unsafe { ::std::mem::uninitialized() };
    unsafe { libglfw3::glfwGetWindowSize(window, &mut width, &mut height) };

    let sender : &Sender<WindowEvent> = unsafe { transmute(libglfw3::glfwGetWindowUserPointer(window)) };
    let x = (2.0 * x / (width as f64)) - 1.0;
    let y = (2.0 * (1.0 - (y / (height as f64)))) - 1.0;
    sender.send(super::super::Input(::input::MouseMoved(x as f64, y as f64)));
}

impl Window {
    pub fn new(width: uint, height: uint, title: &str) -> Window {
        unsafe {
            GLFWInitialized.doit(|| {
                if libglfw3::glfwInit() == 0 {
                    fail!("glfwInit failed");
                }
                std::rt::at_exit(proc() {
                    libglfw3::glfwTerminate();
                });
                libglfw3::glfwSetErrorCallback(Some(errorCallback));
            })
        }

        let commands = CommandsThread::new();
        let sharedTitle = String::from_str(title);

        let (tx, rx) = channel();
        let txBox = box tx;

        let mut handle = unsafe {
            let txRef: &Sender<WindowEvent> = txBox;
            let txPtr: *const Sender<WindowEvent> = transmute(txRef);

            commands.exec(proc() {
                let handle = sharedTitle.with_c_str(|title| {
                    libglfw3::glfwCreateWindow(width as c_int, height as c_int, title, null(), null())
                });

                if handle.is_null() {
                    fail!("glfwCreateWindow failed");
                }

                libglfw3::glfwSetWindowUserPointer(handle, transmute(txPtr));

                libglfw3::glfwSetKeyCallback(handle, Some(keyCallback));
                libglfw3::glfwSetWindowPosCallback(handle, Some(posCallback));
                libglfw3::glfwSetWindowSizeCallback(handle, Some(sizeCallback));
                libglfw3::glfwSetWindowCloseCallback(handle, Some(closeCallback));
                libglfw3::glfwSetCursorPosCallback(handle, Some(cursorPosCallback));

                handle
            })
        }.get();

        Window {
            handle: handle,
            commands: commands,
            eventsSender: txBox,
            eventsReceiver: rx
        }
    }

    pub fn exec<T:Send>(&self, f: proc(): Send -> T) -> Future<T> {
        self.commands.exec(f)
    }

    pub fn swap_buffers(&self) {
        let handle = self.handle;
        self.commands.exec(proc() {
            unsafe { libglfw3::glfwSwapBuffers(handle) }
        });
    }

    pub fn recv(&self) -> Option<WindowEvent> {
        self.commands.exec(proc() {
            unsafe { libglfw3::glfwPollEvents(); }
        }).get();
        
        match self.eventsReceiver.try_recv() {
            Ok(val) => Some(val),
            Err(Empty) => None,
            Err(Disconnected) => fail!()
        }
    }

    pub fn make_context_current(&self) {
        let handle = self.handle;
        self.commands.exec(proc() {
            unsafe { libglfw3::glfwMakeContextCurrent(handle) }
        }).get();
    }

    pub fn get_cursor_pos(&self)
        -> (f64, f64)
    {
        let mut x: ::libc::c_double = unsafe { ::std::mem::uninitialized() };
        let mut y: ::libc::c_double = unsafe { ::std::mem::uninitialized() };
        unsafe { libglfw3::glfwGetCursorPos(self.handle, &mut x, &mut y) };
        (x as f64, y as f64)
    }
}
