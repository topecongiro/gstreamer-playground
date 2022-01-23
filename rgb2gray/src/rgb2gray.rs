use gst::glib;
use gst::prelude::*;
use gst::Plugin;

mod imp;

glib::wrapper! {
    pub struct Rgb2Gray(ObjectSubclass<imp::Rgb2Gray>) @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

unsafe impl Send for Rgb2Gray {}
unsafe impl Sync for Rgb2Gray {}

pub fn register(plugin: &Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        "rsrgb2gray",
        gst::Rank::None,
        Rgb2Gray::static_type(),
    )
}
