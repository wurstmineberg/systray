use {
    std::io,
    winres::WindowsResource,
};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=assets/manifest.xml");
    WindowsResource::new()
        .set_icon("assets/wurstpick.ico")
        .set_manifest_file("assets/manifest.xml")
        .set_icon_with_id("assets/logo-black-16.ico", "2")
        .set_icon_with_id("assets/logo-black-32.ico", "3")
        .set_icon_with_id("assets/logo-white-16.ico", "4")
        .set_icon_with_id("assets/logo-white-32.ico", "5")
        .compile()?;
    Ok(())
}
