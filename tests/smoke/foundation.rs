use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use audiacore_core::{Application, ApplicationId, CorrelationId};
use audiacore_events::{EventId, EventPolicy, EventStream, EventStreamId};
use audiacore_host::{FileReadAuthority, FileWriteAuthority};
use audiacore_host_native::NativeFileHost;
use audiacore_managed_content::{
    ManagedContentApplyResult, ManagedContentTarget, apply, observe, plan,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SmokeEvent {
    Applied(ManagedContentApplyResult),
}

struct SmokeComposition {
    host: NativeFileHost,
    read_authority: FileReadAuthority,
    write_authority: FileWriteAuthority,
    target: ManagedContentTarget,
    events: EventStream<SmokeEvent>,
}

struct TempRoot(PathBuf);

impl TempRoot {
    fn new() -> Result<Self, Box<dyn Error>> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!(
            "audiacore-foundation-smoke-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path)?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn foundation_composes_across_crates() -> Result<(), Box<dyn Error>> {
    let root = TempRoot::new()?;
    let target_path = root.path().join("state.bin");

    let composition = SmokeComposition {
        host: NativeFileHost,
        read_authority: FileReadAuthority::new(root.path())?,
        write_authority: FileWriteAuthority::new(root.path())?,
        target: ManagedContentTarget::new(&target_path),
        events: EventStream::new(
            EventStreamId::new("foundation-smoke")?,
            EventPolicy::unbounded(),
        ),
    };

    let mut application =
        Application::new(ApplicationId::new("audiacore-smoke")?, composition);
    let correlation = CorrelationId::new("smoke-run")?;
    let desired = Some(b"validated".to_vec());

    {
        let composition = application.composition_mut();

        let observed = observe(
            &composition.host,
            &composition.read_authority,
            &composition.target,
        )?;
        assert_eq!(observed, None);

        let create = plan(&composition.target, &observed, &desired);
        let first = apply(
            &composition.host,
            &composition.write_authority,
            &create,
        )?;
        assert_eq!(first, ManagedContentApplyResult::Created);

        let first_sequence = composition.events.append(
            EventId::new("apply-created")?,
            correlation.clone(),
            None,
            SmokeEvent::Applied(first),
        )?;
        assert_eq!(first_sequence.get(), 1);

        let observed = observe(
            &composition.host,
            &composition.read_authority,
            &composition.target,
        )?;
        assert_eq!(observed, desired);

        let noop = plan(&composition.target, &observed, &desired);
        let second = apply(
            &composition.host,
            &composition.write_authority,
            &noop,
        )?;
        assert_eq!(second, ManagedContentApplyResult::Noop);

        let second_sequence = composition.events.append(
            EventId::new("apply-noop")?,
            correlation,
            None,
            SmokeEvent::Applied(second),
        )?;
        assert_eq!(second_sequence.get(), 2);
    }

    assert_eq!(application.id().as_str(), "audiacore-smoke");
    assert_eq!(fs::read(&target_path)?, b"validated");

    let events = &application.composition().events;
    assert_eq!(events.len(), 2);
    assert_eq!(events.last_sequence().map(|sequence| sequence.get()), Some(2));

    let payloads = events.iter().map(|event| *event.payload()).collect::<Vec<_>>();
    assert_eq!(
        payloads,
        vec![
            SmokeEvent::Applied(ManagedContentApplyResult::Created),
            SmokeEvent::Applied(ManagedContentApplyResult::Noop),
        ]
    );

    Ok(())
}
