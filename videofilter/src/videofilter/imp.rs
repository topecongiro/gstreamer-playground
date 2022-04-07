use gst::{Buffer, BufferRef, Caps, ErrorMessage, Event, FlowError, FlowSuccess, glib, gst_info, gst_warning, LoggableError, Meta, MetaRef, PadDirection, PadTemplate, QueryRef};
use gst::query::Allocation;
use gst::subclass::ElementMetadata;
use gst::subclass::prelude::*;
use gst::prelude::*;
use gst_base::subclass::{*, prelude::*};
use gst_base::subclass::base_transform::{GenerateOutputSuccess, InputBuffer, PrepareOutputBufferSuccess};
use parking_lot::Mutex;
use crate::glib::once_cell::sync::Lazy;

fn get_all_video_formats() -> Vec<glib::SendValue> {
    use gst_video::VideoFormat;

    let values = [
        VideoFormat::Gray8,
        VideoFormat::Gray16Le,
        VideoFormat::Gray16Be,
        VideoFormat::Y444,
        VideoFormat::Y44410le,
        VideoFormat::Y44410be,
        VideoFormat::A44410le,
        VideoFormat::A44410be,
        VideoFormat::A420,
        VideoFormat::Y42b,
        VideoFormat::I42210le,
        VideoFormat::I42210be,
        VideoFormat::A42210le,
        VideoFormat::A42210be,
        VideoFormat::I420,
        VideoFormat::I42010le,
        VideoFormat::I42010be,
        VideoFormat::Gbr,
        VideoFormat::Gbr10le,
        VideoFormat::Gbr10be,
        VideoFormat::Y41b,
        VideoFormat::Yuv9,
    ];

    values.iter().map(|i| i.to_str().to_send_value()).collect()
}

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "videofilter",
        gst::DebugColorFlags::empty(),
        Some("Video filter"),
    )
});

struct State {
}

#[derive(Default)]
pub struct VideoFilter {
    state: Mutex<Option<State>>,
}

#[glib::object_subclass]
impl ObjectSubclass for VideoFilter {
    const NAME: &'static str = "VideoFilter";
    type Type = super::VideoFilter;
    type ParentType = gst_base::BaseTransform;
}

impl ElementImpl for VideoFilter {
    fn metadata() -> Option<&'static ElementMetadata> {
        static METADATA: Lazy<ElementMetadata> = Lazy::new(|| {
            ElementMetadata::new(
                "Video filter",
                "Filter",
                "View video data",
                "Seiichi Uchida <topecongiro@fastmail.com>",
            )
        });

        Some(&*METADATA)
    }

    fn pad_templates() -> &'static [PadTemplate] {
        static PAD_TEMPLATE: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            let sink_caps = gst::Caps::builder("video/x-raw")
                .field("width", gst::IntRange::new(1, i32::MAX))
                .field("height", gst::IntRange::new(1, i32::MAX))
                .field("framerate", gst::FractionRange::new(
                    gst::Fraction::new(0, 1),
                    gst::Fraction::new(i32::MAX, 1),
                ))
                .build();
            let sink_pad_template = gst::PadTemplate::new(
                "sink",
                gst::PadDirection::Sink,
                gst::PadPresence::Always,
                &sink_caps
            ).unwrap();

            let src_caps = gst::Caps::builder("video/x-raw")
                .field("format", gst::List::from(get_all_video_formats()))
                .field("width", gst::IntRange::new(1, i32::MAX))
                .field("height", gst::IntRange::new(1, i32::MAX))
                .field("framerate", gst::FractionRange::new(
                    gst::Fraction::new(0, 1),
                    gst::Fraction::new(i32::MAX, 1),
                ))
                .build();
            let src_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &src_caps,
            ).unwrap();

            vec![sink_pad_template, src_pad_template]
        });

        PAD_TEMPLATE.as_ref()
    }
}

impl BaseTransformImpl for VideoFilter {
    const MODE: BaseTransformMode = BaseTransformMode::Both;
    const PASSTHROUGH_ON_SAME_CAPS: bool = true;
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = false;

    fn transform_caps(&self, element: &Self::Type, direction: PadDirection, caps: &Caps, filter: Option<&Caps>) -> Option<Caps> {
        let other_caps = caps.clone();
        gst_warning!(CAT, "transform_ip: {:?} {:?}", caps, element);
        Some(other_caps)
    }

    fn transform_ip_passthrough(&self, element: &Self::Type, buf: &Buffer) -> Result<FlowSuccess, FlowError> {
        gst_warning!(CAT, "transform_ip_passthrough: {:?} {:?}", element, buf);
        Ok(FlowSuccess::Ok)
    }
}

impl GstObjectImpl for VideoFilter {}

impl ObjectImpl for VideoFilter {}