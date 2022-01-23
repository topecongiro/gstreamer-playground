use gst::glib;
use gst::glib::once_cell::sync::Lazy;
use gst::prelude::ToValue;
use gst::subclass::prelude::*;
use gst::{gst_debug, gst_info};
use gst_base::subclass::prelude::*;
use gst_base::subclass::BaseTransformMode;
use parking_lot::Mutex;

const DEFAULT_INVERT: bool = false;
const DEFAULT_SHIFT: u32 = 0;

#[derive(Debug, Clone, Copy)]
struct Settings {
    invert: bool,
    shift: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            invert: DEFAULT_INVERT, shift: DEFAULT_SHIFT }
    }
}

struct State {
    in_info: gst_video::VideoInfo,
    out_info: gst_video::VideoInfo,
}

#[derive(Default)]
pub struct Rgb2Gray {
    settings: Mutex<Settings>,
    state: Mutex<Option<State>>,
}

impl Rgb2Gray {
    #[inline]
    fn bgrx_to_gray(in_p: &[u8], shift: u8, invert: bool) -> u8 {
        // See https://en.wikipedia.org/wiki/YUV#SDTV_with_BT.601
        const R_Y: u32 = 19595; // 0.299 * 65536
        const G_Y: u32 = 38470; // 0.587 * 65536
        const B_Y: u32 = 7471; // 0.114 * 65536

        assert_eq!(in_p.len(), 4);

        let b = u32::from(in_p[0]);
        let g = u32::from(in_p[1]);
        let r = u32::from(in_p[2]);

        let gray = ((r * R_Y) + (g * G_Y) + (b * B_Y)) / 65536;
        let gray = (gray as u8).wrapping_add(shift);

        if invert {
            255 - gray
        } else {
            gray
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for Rgb2Gray {
    const NAME: &'static str = "RsRgb2Gray";
    type Type = super::Rgb2Gray;
    type ParentType = gst_base::BaseTransform;

    fn new() -> Self {
        Self::default()
    }
}

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "RsRgb2Gray",
        gst::DebugColorFlags::empty(),
        Some("Rust RGB-GRAY converter"),
    )
});

impl GstObjectImpl for Rgb2Gray {}

impl ObjectImpl for Rgb2Gray {
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecBoolean::new(
                    "invert",
                    "invert",
                    "Invert grayscale output",
                    DEFAULT_INVERT,
                    glib::ParamFlags::READWRITE | gst::PARAM_FLAG_MUTABLE_PLAYING,
                ),
                glib::ParamSpecUInt::new(
                    "shift",
                    "shift",
                    "Shift grayscale output (wrapping around)",
                    0,
                    255,
                    DEFAULT_SHIFT,
                    glib::ParamFlags::READWRITE | gst::PARAM_FLAG_MUTABLE_PLAYING,
                ),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn set_property(&self, obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "invert" => {
                let mut settings = self.settings.lock();
                let invert = value.get().unwrap();
                gst_info!(
                    CAT,
                    obj: obj,
                    "Changing invert from {} to {}",
                    settings.invert, invert
                );
                settings.invert = invert;
            }
            "shift" => {
                let mut settings = self.settings.lock();
                let shift = value.get().unwrap();
                gst_info!(
                    CAT,
                    obj: obj,
                    "Changing shift from {} to {}",
                    settings.shift, shift
                );
                settings.shift = shift;
            }
            _ => unimplemented!()
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "invert" => {
                let settings = self.settings.lock();
                settings.invert.to_value()
            }
            "shift" => {
                let settings = self.settings.lock();
                settings.shift.to_value()
            }
            _ => unimplemented!(),
        } 
    }
}

impl ElementImpl for Rgb2Gray {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                "RGB-GRAY converter",
                "Filter/Effect/Converter/Video",
                "Converts RGB to GRAY or grayscale RGB",
                "Seiichi Uchida <topecongiro@fastmail.com>",
            )
        });

        Some(&*METADATA)
    }

    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            let caps = gst::Caps::builder("video/x-raw")
                .field(
                    "format",
                    gst::List::new([
                        gst_video::VideoFormat::Bgrx.to_str(),
                        gst_video::VideoFormat::Gray8.to_str(),
                    ]),
                )
                .field("width", gst::IntRange::new(0, i32::MAX))
                .field("height", gst::IntRange::new(0, i32::MAX))
                .field(
                    "framerate",
                    gst::FractionRange::new(
                        gst::Fraction::new(0, 1),
                        gst::Fraction::new(i32::MAX, 1),
                    ),
                )
                .build();

            let src_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            let caps = gst::Caps::builder("video/x-raw")
                .field("format", gst_video::VideoFormat::Bgrx.to_str())
                .field("width", gst::IntRange::new(0, i32::MAX))
                .field("height", gst::IntRange::new(0, i32::MAX))
                .field(
                    "framerate",
                    gst::FractionRange::new(
                        gst::Fraction::new(0, 1),
                        gst::Fraction::new(i32::MAX, 1),
                    ),
                )
                .build();

            let sink_pad_template = gst::PadTemplate::new(
                "sink",
                gst::PadDirection::Sink,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            vec![src_pad_template, sink_pad_template]
        });

        PAD_TEMPLATES.as_ref()
    }
}

impl BaseTransformImpl for Rgb2Gray {
    const MODE: BaseTransformMode = BaseTransformMode::NeverInPlace;

    const PASSTHROUGH_ON_SAME_CAPS: bool = false;

    const TRANSFORM_IP_ON_PASSTHROUGH: bool = false;

    fn set_caps(
        &self,
        element: &Self::Type,
        incaps: &gst::Caps,
        outcaps: &gst::Caps,
    ) -> Result<(), gst::LoggableError> {
        let in_info = gst_video::VideoInfo::from_caps(incaps)
            .map_err(|_| gst::loggable_error!(CAT, "Failed to parse input caps"))?;
        let out_info = gst_video::VideoInfo::from_caps(outcaps)
            .map_err(|_| gst::loggable_error!(CAT, "Failed to parse output caps"))?;

        gst_debug!(
            CAT,
            obj: element,
            "Configured for caps {} to {}",
            incaps,
            outcaps
        );

        *self.state.lock() = Some(State { in_info, out_info });

        Ok(())
    }

    fn stop(&self, element: &Self::Type) -> Result<(), gst::ErrorMessage> {
        // Drop state
        let _ = self.state.lock().take();

        gst_info!(CAT, obj: element, "Stopped");

        Ok(())
    }

    fn unit_size(&self, _element: &Self::Type, caps: &gst::Caps) -> Option<usize> {
        gst_video::VideoInfo::from_caps(caps)
            .as_ref()
            .map(gst_video::VideoInfo::size)
            .ok()
    }

    fn transform_caps(
        &self,
        element: &Self::Type,
        direction: gst::PadDirection,
        caps: &gst::Caps,
        filter: Option<&gst::Caps>,
    ) -> Option<gst::Caps> {
        let other_caps = match direction {
            gst::PadDirection::Src => {
                let mut caps = caps.clone();
                for s in caps.make_mut().iter_mut() {
                    s.set("format", &gst_video::VideoFormat::Bgrx.to_str());
                }
                caps
            }
            gst::PadDirection::Sink => {
                let mut gray_caps = gst::Caps::new_empty();

                {
                    let gray_caps = gray_caps.get_mut()?;
                    for s in caps.iter() {
                        let mut s_gray = s.to_owned();
                        s_gray.set("foramt", gst_video::VideoFormat::Gray8.to_str());
                        gray_caps.append_structure(s_gray);
                    }
                    gray_caps.append(caps.clone())
                }

                gray_caps
            }
            _ => return None,
        };

        gst_debug!(
            CAT,
            obj: element,
            "Transformed caps from {} to {} in direction {:?}",
            caps,
            other_caps,
            direction
        );

        if let Some(filter) = filter {
            Some(filter.intersect_with_mode(&other_caps, gst::CapsIntersectMode::First))
        } else {
            Some(other_caps)
        }
    }

    fn transform(
        &self,
        element: &Self::Type,
        inbuf: &gst::Buffer,
        outbuf: &mut gst::BufferRef,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        let settings = self.settings.lock();
        let mut state_guard = self.state.lock();
        let state = state_guard.as_mut().ok_or_else(|| {
            gst::element_error!(element, gst::CoreError::Negotiation, ["Have no state yet"]);
            gst::FlowError::NotNegotiated
        })?;

        let in_frame =
            gst_video::VideoFrameRef::from_buffer_ref_readable(inbuf.as_ref(), &state.in_info)
                .map_err(|err| {
                    gst::element_error!(
                        element,
                        gst::CoreError::Failed,
                        [&format!("Failed to map input buffer readable: {}", err)]
                    );
                    gst::FlowError::Error
                })?;

        let mut out_frame =
            gst_video::VideoFrameRef::from_buffer_ref_writable(outbuf, &state.out_info).map_err(
                |err| {
                    gst::element_error!(
                        element,
                        gst::CoreError::Failed,
                        [&format!("Failed to map output buffer writable: {}", err)]
                    );
                    gst::FlowError::Error
                },
            )?;

        let width = in_frame.width() as usize;
        let in_stride = in_frame.plane_stride()[0] as usize;
        let in_data = in_frame.plane_data(0).unwrap();
        let out_stride = out_frame.plane_stride()[0] as usize;
        let out_format = out_frame.format();
        let out_data = out_frame.plane_data_mut(0).unwrap();

        if out_format == gst_video::VideoFormat::Bgrx {
            assert_eq!(in_data.len() % 4, 0);
            assert_eq!(out_data.len() % 4, 0);
            assert_eq!(out_data.len() / out_stride, in_data.len() / in_stride);

            let in_line_bytes = width * 4;
            let out_line_bytes = width * 4;

            assert!(in_line_bytes <= in_stride);
            assert!(out_line_bytes <= out_stride);

            for (in_line, out_line) in in_data
                .chunks_exact(in_stride)
                .zip(out_data.chunks_exact_mut(out_stride))
            {
                for (in_p, out_p) in in_line[..in_line_bytes]
                    .chunks_exact(4)
                    .zip(out_line[..out_line_bytes].chunks_exact_mut(4))
                {
                    assert_eq!(out_p.len(), 4);

                    let gray = Rgb2Gray::bgrx_to_gray(in_p, settings.shift as u8, settings.invert);
                    out_p[0] = gray;
                    out_p[1] = gray;
                    out_p[2] = gray;
                }
            }
        } else if out_format == gst_video::VideoFormat::Gray8 {
            assert_eq!(in_data.len() % 4, 0);
            assert_eq!(out_data.len() / out_stride, in_data.len() / in_stride);

            let in_line_bytes = width * 4;
            let out_line_bytes = width;

            assert!(in_line_bytes <= in_stride);
            assert!(out_line_bytes <= out_stride);

            for (in_line, out_line) in in_data
                .chunks_exact(in_stride)
                .zip(out_data.chunks_exact_mut(out_stride))
            {
                for (in_p, out_p) in in_line[..in_line_bytes]
                    .chunks_exact(4)
                    .zip(out_line[..out_line_bytes].iter_mut())
                {
                    let gray = Rgb2Gray::bgrx_to_gray(in_p, settings.shift as u8, settings.invert);
                    *out_p = gray;
                }
            }
        }

        Ok(gst::FlowSuccess::Ok)
    }
}
