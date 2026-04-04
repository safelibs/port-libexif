use core::ffi::c_char;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Message(&'static [u8]);

pub const fn message(bytes: &'static [u8]) -> Message {
    Message(bytes)
}

pub const fn empty_message() -> Message {
    Message(b"\0")
}

impl Message {
    pub const fn as_ptr(self) -> *const c_char {
        self.0.as_ptr().cast()
    }

    pub const fn bytes(self) -> &'static [u8] {
        self.0
    }

    pub const fn is_empty(self) -> bool {
        self.0.len() == 1 && self.0[0] == 0
    }
}

pub const fn gettext(message: Message) -> *const c_char {
    message.as_ptr()
}
