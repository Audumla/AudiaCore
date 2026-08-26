use std::{fs, path::PathBuf};

use audiacore_error_catalog::ErrorCatalogue;

#[test]
fn every_owner_local_catalogue_is_valid_and_codes_are_unique() {
    let crates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let mut paths = fs::read_dir(crates_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("errors.yaml"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    paths.sort();

    assert!(
        !paths.is_empty(),
        "workspace contains no owner-local error catalogues"
    );

    let mut catalogue = ErrorCatalogue::new();
    for path in paths {
        let source = path.to_string_lossy().into_owned();
        let yaml = fs::read_to_string(&path).unwrap();
        catalogue.register_yaml(source, &yaml).unwrap();
    }
}
