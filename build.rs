fn main() {
    if cfg!(target_os = "windows") {
        embed_resource::compile("src/assets/manifest.rc", embed_resource::NONE)
            .manifest_required()
            .unwrap();
    }
    slint_build::compile("ui/main.slint").unwrap();
}
