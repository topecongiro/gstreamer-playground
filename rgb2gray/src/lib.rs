use gst::glib;

mod rgb2gray;

fn plugin_init(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    rgb2gray::register(plugin)?;
    Ok(())
}

gst::plugin_define!(
    rgb2gray,
    env!("CARGO_PKG_DESCRIPTION"),
    plugin_init,
    concat!(env!("CARGO_PKG_VERSION"), "-", env!("COMMIT_ID")),
    "MIT",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_REPOSITORY"),
    env!("BUILD_REL_DATE")
);
