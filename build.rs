use {
    std::io,
    winres::WindowsResource,
};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=assets/manifest.xml");
    WindowsResource::new()
        .set_icon_with_id("assets/wurstpick.ico", "ICON")
        .set_manifest_file("assets/manifest.xml")
        .set_icon_with_id("assets/logo-black-16.ico", "LOGO_BLACK_16")
        .set_icon_with_id("assets/logo-black-32.ico", "LOGO_BLACK_32")
        .set_icon_with_id("assets/logo-white-16.ico", "LOGO_WHITE_16")
        .set_icon_with_id("assets/logo-white-32.ico", "LOGO_WHITE_32")
        .compile()?;
    Ok(())
}
