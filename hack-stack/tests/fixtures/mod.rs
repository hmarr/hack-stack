use hack_stack::common;
use std::path::PathBuf;

pub fn load(path_parts: &[&str]) -> common::SourceFile {
    let root_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = &["tests", "fixtures"]
        .iter()
        .chain(path_parts)
        .fold(root_dir, |path, &part| path.join(part));
    let src = std::fs::read_to_string(fixture_path).unwrap();
    common::SourceFile::new(src, path_parts.last().unwrap().to_string())
}
