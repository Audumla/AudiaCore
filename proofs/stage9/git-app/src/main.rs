use std::{error::Error, path::PathBuf};

use audiacore_core::{Application, ApplicationId};
use audiacore_host::{FileReadAuthority, FileWriteAuthority};
use audiacore_managed_content::{
    ManagedContentApplyResult, ManagedContentTarget, apply, observe, plan,
};
use audiacore_stage9_memory_host::MemoryFileHost;

struct Composition {
    host: MemoryFileHost,
    read: FileReadAuthority,
    write: FileWriteAuthority,
}

#[cfg(windows)]
fn authority_root() -> PathBuf {
    PathBuf::from(r"C:\audiacore-stage9-proof")
}

#[cfg(not(windows))]
fn authority_root() -> PathBuf {
    PathBuf::from("/audiacore-stage9-proof")
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = authority_root();
    let application = Application::new(
        ApplicationId::new("stage9-source-proof")?,
        Composition {
            host: MemoryFileHost::new(),
            read: FileReadAuthority::new(root.clone())?,
            write: FileWriteAuthority::new(root.clone())?,
        },
    );
    let composition = application.composition();
    let target = ManagedContentTarget::new(root.join("proof.txt"));
    let desired = Some(b"stage9-source-proof".to_vec());

    let observed = observe(&composition.host, &composition.read, &target)?;
    let first = apply(
        &composition.host,
        &composition.write,
        &plan(&target, &observed, &desired),
    )?;
    assert_eq!(first, ManagedContentApplyResult::Created);

    let observed = observe(&composition.host, &composition.read, &target)?;
    let second = apply(
        &composition.host,
        &composition.write,
        &plan(&target, &observed, &desired),
    )?;
    assert_eq!(second, ManagedContentApplyResult::Noop);
    assert_eq!(
        composition.host.bytes(target.path()).as_deref(),
        Some(b"stage9-source-proof".as_slice())
    );

    println!("AUDIACORE_STAGE9_SOURCE_OK first=created second=noop");
    Ok(())
}
