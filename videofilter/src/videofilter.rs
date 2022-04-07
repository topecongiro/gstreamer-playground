use gst::glib::{self, StaticType};

mod imp;

glib::wrapper! {
    pub struct VideoFilter(ObjectSubclass<imp::VideoFilter>) @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

unsafe impl Send for VideoFilter {}
unsafe impl Sync for VideoFilter {}

pub(crate) fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        "videofilter",
        gst::Rank::None,
        VideoFilter::static_type(),
    )
}