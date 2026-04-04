pub mod ffi;

mod i18n;
mod mnote;
mod object;
mod parser;
mod primitives;
mod runtime;
mod tables;

pub use mnote::base::{
    exif_mnote_data_construct, exif_mnote_data_count, exif_mnote_data_get_description,
    exif_mnote_data_get_id, exif_mnote_data_get_name, exif_mnote_data_get_title,
    exif_mnote_data_get_value, exif_mnote_data_load, exif_mnote_data_log, exif_mnote_data_ref,
    exif_mnote_data_save, exif_mnote_data_set_byte_order, exif_mnote_data_set_offset,
    exif_mnote_data_unref,
};
pub use mnote::canon::{
    exif_mnote_data_canon_new, mnote_canon_entry_get_value, mnote_canon_tag_get_description,
    mnote_canon_tag_get_name, mnote_canon_tag_get_title,
};
pub use mnote::olympus::{
    exif_mnote_data_olympus_new, mnote_olympus_entry_get_value, mnote_olympus_tag_get_description,
    mnote_olympus_tag_get_name, mnote_olympus_tag_get_title,
};
pub use mnote::pentax::{
    exif_mnote_data_pentax_new, mnote_pentax_entry_get_value, mnote_pentax_tag_get_description,
    mnote_pentax_tag_get_name, mnote_pentax_tag_get_title,
};

unsafe extern "C" {
    fn exif_log_shim_anchor();
}

#[used]
static FORCE_EXIF_LOG_SHIM: unsafe extern "C" fn() = exif_log_shim_anchor;
