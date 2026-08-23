use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

use cap_std::{
    ambient_authority,
    fs::{Dir, OpenOptions},
};

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn test_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "audiacore-cap-std-proof-{}-{name}",
        std::process::id()
    ))
}

fn open_root(path: &Path) -> io::Result<Dir> {
    Dir::open_ambient_dir(path, ambient_authority())
}

fn write_atomic(dir: &Dir, target: &Path, bytes: &[u8]) -> io::Result<()> {
    let name = target
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target has no file name"))?;
    let mut temp_name = std::ffi::OsString::from(".");
    temp_name.push(name);
    temp_name.push(".proof-tmp");
    let temp = target.with_file_name(temp_name);

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = dir.open_with(&temp, &options)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);

    dir.rename(&temp, dir, target)
}

#[test]
fn capability_relative_atomic_create_replace_read_and_remove_work() {
    let _guard = TEST_LOCK.lock().unwrap();
    let root = test_root("round-trip");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let dir = open_root(&root).unwrap();
    write_atomic(&dir, Path::new("state.bin"), b"one").unwrap();
    assert_eq!(dir.read("state.bin").unwrap(), b"one");

    write_atomic(&dir, Path::new("state.bin"), b"two").unwrap();
    assert_eq!(dir.read("state.bin").unwrap(), b"two");

    dir.remove_file("state.bin").unwrap();
    assert_eq!(
        dir.read("state.bin").unwrap_err().kind(),
        io::ErrorKind::NotFound
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn parent_component_cannot_escape_directory_capability() {
    let _guard = TEST_LOCK.lock().unwrap();
    let root = test_root("escape-root");
    let outside = test_root("escape-outside");
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("state.bin"), b"outside").unwrap();

    let escaped = PathBuf::from("..")
        .join(outside.file_name().unwrap())
        .join("state.bin");
    let dir = open_root(&root).unwrap();

    assert!(dir.read(&escaped).is_err());

    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    assert!(dir.open_with(&escaped, &options).is_err());
    assert_eq!(fs::read(outside.join("state.bin")).unwrap(), b"outside");

    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[cfg(unix)]
#[test]
fn directory_symlink_cannot_escape_directory_capability() {
    use std::os::unix::fs::symlink;

    let _guard = TEST_LOCK.lock().unwrap();
    let root = test_root("symlink-root");
    let outside = test_root("symlink-outside");
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("state.bin"), b"outside").unwrap();
    symlink(&outside, root.join("escape")).unwrap();

    let dir = open_root(&root).unwrap();
    assert!(dir.read("escape/state.bin").is_err());

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    assert!(dir.open_with("escape/new.bin", &options).is_err());
    assert_eq!(fs::read(outside.join("state.bin")).unwrap(), b"outside");
    assert!(!outside.join("new.bin").exists());

    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[cfg(unix)]
#[test]
fn leaf_symlink_is_visible_without_following_for_write_policy() {
    use std::os::unix::fs::symlink;

    let _guard = TEST_LOCK.lock().unwrap();
    let root = test_root("leaf-root");
    let outside = test_root("leaf-outside");
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let outside_file = outside.join("state.bin");
    fs::write(&outside_file, b"outside").unwrap();
    symlink(&outside_file, root.join("state.bin")).unwrap();

    let dir = open_root(&root).unwrap();
    let metadata = dir.symlink_metadata("state.bin").unwrap();
    assert!(metadata.file_type().is_symlink());
    assert!(dir.read("state.bin").is_err());
    assert_eq!(fs::read(outside_file).unwrap(), b"outside");

    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}
