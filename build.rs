fn main() {
    if cfg!(target_os = "windows") {
        embed_resource::compile("manifest.rc", embed_resource::NONE)
            .manifest_required()
            .unwrap();
    }
}
