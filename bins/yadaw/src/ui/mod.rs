use kui::elements::text::TextResource;

pub mod components;
pub mod magic_menu;

/// Initializes the fonts for the application.
pub fn initialize_fonts(ctx: &kui::Ctx) -> std::io::Result<()> {
    const SUPPORTED_EXTENSIONS: &[&[u8]] = &[b"ttf"];

    ctx.with_resource_or_default(|res: &mut TextResource| {
        for entry in std::fs::read_dir("bins/yadaw/assets/fonts")? {
            let entry = entry?;

            if !entry.file_type()?.is_file() {
                continue;
            }

            let path = entry.path();

            let ext = path.extension().unwrap_or_default().as_encoded_bytes();
            if !SUPPORTED_EXTENSIONS.contains(&ext) {
                continue;
            }

            res.register_font(std::fs::read(path)?);
        }
        Ok(())
    })
}
