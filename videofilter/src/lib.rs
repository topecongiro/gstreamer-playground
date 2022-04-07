use gst::glib;

mod videofilter;

fn plugin_init(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    videofilter::register(plugin)?;
    Ok(())
}

gst::plugin_define!(
    videofilter,
    env!("CARGO_PKG_DESCRIPTION"),
    plugin_init,
    concat!(env!("CARGO_PKG_VERSION"), "-", env!("COMMIT_ID")),
    "MIT/X11",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_REPOSITORY"),
    env!("BUILD_REL_DATE")
);
