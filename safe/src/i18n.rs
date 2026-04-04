use core::ffi::c_char;
use std::sync::Once;

const GETTEXT_PACKAGE: &[u8] = concat!(env!("LIBEXIF_GETTEXT_PACKAGE"), "\0").as_bytes();
const LOCALEDIR: &[u8] = concat!(env!("LIBEXIF_LOCALEDIR"), "\0").as_bytes();
const UTF8: &[u8] = b"UTF-8\0";

static INIT_GETTEXT: Once = Once::new();

unsafe extern "C" {
    fn bindtextdomain(domainname: *const c_char, dirname: *const c_char) -> *mut c_char;
    fn bind_textdomain_codeset(domainname: *const c_char, codeset: *const c_char) -> *mut c_char;
    fn dgettext(domainname: *const c_char, message: *const c_char) -> *mut c_char;
}

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

fn init_gettext() {
    INIT_GETTEXT.call_once(|| unsafe {
        bindtextdomain(GETTEXT_PACKAGE.as_ptr().cast(), LOCALEDIR.as_ptr().cast());
        bind_textdomain_codeset(GETTEXT_PACKAGE.as_ptr().cast(), UTF8.as_ptr().cast());
    });
}

pub fn gettext(message: Message) -> *const c_char {
    init_gettext();

    let translated = unsafe { dgettext(GETTEXT_PACKAGE.as_ptr().cast(), message.as_ptr()) };
    if translated.is_null() {
        message.as_ptr()
    } else {
        translated.cast_const()
    }
}
