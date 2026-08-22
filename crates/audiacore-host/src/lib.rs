//! Narrow host-facility contracts with explicit authority scopes.
//!
//! This crate defines permission-bearing boundaries only. It never performs
//! native I/O and does not aggregate facilities into a global host object.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    ffi::{OsStr, OsString},
    fmt,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use audiacore_errors::{CodedError, ErrorCode, ErrorDefinition};
use audiacore_sensitive::Sensitive;

const FILE_ROOT_NOT_ABSOLUTE: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("VAL-HOST-FILE-001"),
    "File authority root must be absolute.",
    "Resolve the authority root to an absolute application-owned path before granting access.",
);
const PROCESS_PROGRAM_NOT_ABSOLUTE: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("VAL-HOST-PROCESS-001"),
    "Process program path must be absolute.",
    "Resolve executable paths to absolute application-owned values before granting or requesting launch.",
);
const PROCESS_CURRENT_DIR_NOT_ABSOLUTE: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("VAL-HOST-PROCESS-002"),
    "Process working directory must be absolute.",
    "Resolve the child working directory to an absolute path before building the process request.",
);
const PROCESS_ENVIRONMENT_KEY_EMPTY: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("VAL-HOST-PROCESS-003"),
    "Process environment key must not be empty.",
    "Provide a non-empty environment variable key.",
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileAuthorityError {
    RootNotAbsolute(PathBuf),
}

impl CodedError for FileAuthorityError {
    fn definition(&self) -> &'static ErrorDefinition {
        match self {
            Self::RootNotAbsolute(_) => &FILE_ROOT_NOT_ABSOLUTE,
        }
    }
}

impl fmt::Display for FileAuthorityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootNotAbsolute(path) => {
                write!(f, "file authority root must be absolute: {path:?}")
            }
        }
    }
}

impl Error for FileAuthorityError {}

/// Permission to observe files beneath one explicit root.
///
/// This type intentionally exposes no lexical `allows(path)` helper. Safe
/// containment depends on canonicalization and symlink-aware checks performed
/// by the concrete host implementation at the effect boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileReadAuthority {
    root: PathBuf,
}

impl FileReadAuthority {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, FileAuthorityError> {
        let root = root.into();
        validate_file_root(&root)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

/// Permission to create, replace or remove files beneath one explicit root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileWriteAuthority {
    root: PathBuf,
}

impl FileWriteAuthority {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, FileAuthorityError> {
        let root = root.into();
        validate_file_root(&root)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

fn validate_file_root(root: &Path) -> Result<(), FileAuthorityError> {
    if root.is_absolute() {
        Ok(())
    } else {
        Err(FileAuthorityError::RootNotAbsolute(root.to_path_buf()))
    }
}

/// Filesystem effect boundary required by managed configuration.
///
/// Mandatory-read, directory traversal, watching and metadata operations are
/// deliberately absent until a real consumer proves those semantics.
pub trait FileHost: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    fn read_optional(
        &self,
        authority: &FileReadAuthority,
        path: &Path,
    ) -> Result<Option<Vec<u8>>, Self::Error>;

    fn write(
        &self,
        authority: &FileWriteAuthority,
        path: &Path,
        bytes: &[u8],
    ) -> Result<(), Self::Error>;

    fn remove(&self, authority: &FileWriteAuthority, path: &Path) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessContractError {
    ProgramNotAbsolute(PathBuf),
    CurrentDirNotAbsolute(PathBuf),
    EmptyEnvironmentKey,
}

impl CodedError for ProcessContractError {
    fn definition(&self) -> &'static ErrorDefinition {
        match self {
            Self::ProgramNotAbsolute(_) => &PROCESS_PROGRAM_NOT_ABSOLUTE,
            Self::CurrentDirNotAbsolute(_) => &PROCESS_CURRENT_DIR_NOT_ABSOLUTE,
            Self::EmptyEnvironmentKey => &PROCESS_ENVIRONMENT_KEY_EMPTY,
        }
    }
}

impl fmt::Display for ProcessContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProgramNotAbsolute(path) => {
                write!(f, "process program path must be absolute: {path:?}")
            }
            Self::CurrentDirNotAbsolute(path) => {
                write!(f, "process working directory must be absolute: {path:?}")
            }
            Self::EmptyEnvironmentKey => f.write_str("process environment key must not be empty"),
        }
    }
}

impl Error for ProcessContractError {}

/// Permission to launch a bounded set of executable paths.
///
/// This is launch authority only. It does not sandbox the resulting process or
/// constrain the operating-system authority inherited by the child account.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessAuthority {
    programs: BTreeSet<PathBuf>,
}

impl ProcessAuthority {
    pub fn new(programs: impl IntoIterator<Item = PathBuf>) -> Result<Self, ProcessContractError> {
        let mut validated = BTreeSet::new();
        for program in programs {
            validate_program_path(&program)?;
            validated.insert(program);
        }
        Ok(Self {
            programs: validated,
        })
    }

    pub fn programs(&self) -> &BTreeSet<PathBuf> {
        &self.programs
    }
}

fn validate_program_path(program: &Path) -> Result<(), ProcessContractError> {
    if program.is_absolute() {
        Ok(())
    } else {
        Err(ProcessContractError::ProgramNotAbsolute(
            program.to_path_buf(),
        ))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProcessStdio {
    #[default]
    Pipe,
    Null,
    Inherit,
}

/// Description of one child launch.
///
/// Environment values are sensitive by construction. Ambient environment
/// inheritance is disabled by default and must be explicitly requested.
pub struct ProcessRequest {
    program: PathBuf,
    args: Vec<OsString>,
    current_dir: Option<PathBuf>,
    environment: BTreeMap<OsString, Sensitive<OsString>>,
    inherit_environment: bool,
    stdin: ProcessStdio,
    stdout: ProcessStdio,
    stderr: ProcessStdio,
}

impl ProcessRequest {
    pub fn new(program: impl Into<PathBuf>) -> Result<Self, ProcessContractError> {
        let program = program.into();
        validate_program_path(&program)?;
        Ok(Self {
            program,
            args: Vec::new(),
            current_dir: None,
            environment: BTreeMap::new(),
            inherit_environment: false,
            stdin: ProcessStdio::Pipe,
            stdout: ProcessStdio::Pipe,
            stderr: ProcessStdio::Pipe,
        })
    }

    pub fn program(&self) -> &Path {
        &self.program
    }

    pub fn args(&self) -> &[OsString] {
        &self.args
    }

    pub fn current_dir(&self) -> Option<&Path> {
        self.current_dir.as_deref()
    }

    pub fn environment(&self) -> impl Iterator<Item = (&OsStr, &Sensitive<OsString>)> {
        self.environment
            .iter()
            .map(|(key, value)| (key.as_os_str(), value))
    }

    pub const fn inherits_environment(&self) -> bool {
        self.inherit_environment
    }

    pub const fn stdin_mode(&self) -> ProcessStdio {
        self.stdin
    }

    pub const fn stdout_mode(&self) -> ProcessStdio {
        self.stdout
    }

    pub const fn stderr_mode(&self) -> ProcessStdio {
        self.stderr
    }

    pub fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args_from(mut self, args: impl IntoIterator<Item = OsString>) -> Self {
        self.args.extend(args);
        self
    }

    pub fn current_dir_path(
        mut self,
        path: impl Into<PathBuf>,
    ) -> Result<Self, ProcessContractError> {
        let path = path.into();
        if !path.is_absolute() {
            return Err(ProcessContractError::CurrentDirNotAbsolute(path));
        }
        self.current_dir = Some(path);
        Ok(self)
    }

    pub fn env_secret(
        mut self,
        key: impl Into<OsString>,
        value: Sensitive<OsString>,
    ) -> Result<Self, ProcessContractError> {
        let key = key.into();
        if key.as_os_str().is_empty() {
            return Err(ProcessContractError::EmptyEnvironmentKey);
        }
        self.environment.insert(key, value);
        Ok(self)
    }

    pub const fn inherit_environment(mut self, inherit: bool) -> Self {
        self.inherit_environment = inherit;
        self
    }

    pub const fn stdin(mut self, mode: ProcessStdio) -> Self {
        self.stdin = mode;
        self
    }

    pub const fn stdout(mut self, mode: ProcessStdio) -> Self {
        self.stdout = mode;
        self
    }

    pub const fn stderr(mut self, mode: ProcessStdio) -> Self {
        self.stderr = mode;
        self
    }
}

impl fmt::Debug for ProcessRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProcessRequest")
            .field("program", &self.program)
            .field("args", &self.args)
            .field("current_dir", &self.current_dir)
            .field(
                "environment_keys",
                &self.environment.keys().collect::<Vec<_>>(),
            )
            .field("inherit_environment", &self.inherit_environment)
            .field("stdin", &self.stdin)
            .field("stdout", &self.stdout)
            .field("stderr", &self.stderr)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessExit {
    code: Option<i32>,
    success: bool,
}

impl ProcessExit {
    pub const fn new(code: Option<i32>, success: bool) -> Self {
        Self { code, success }
    }

    pub const fn code(self) -> Option<i32> {
        self.code
    }

    pub const fn success(self) -> bool {
        self.success
    }
}

/// Owned child-process lifecycle. Stream access is ownership-based only; the
/// application/runtime decides whether to read/write those streams directly or
/// adapt them onto threads or an async reactor.
pub trait ProcessChild: Send {
    type Error: Error + Send + Sync + 'static;

    fn id(&self) -> u32;
    fn take_stdin(&mut self) -> Option<Box<dyn Write + Send>>;
    fn take_stdout(&mut self) -> Option<Box<dyn Read + Send>>;
    fn take_stderr(&mut self) -> Option<Box<dyn Read + Send>>;
    fn try_wait(&mut self) -> Result<Option<ProcessExit>, Self::Error>;
    fn wait(&mut self) -> Result<ProcessExit, Self::Error>;
    fn kill(&mut self) -> Result<(), Self::Error>;

    fn close_stdin(&mut self) {
        drop(self.take_stdin());
    }

    fn is_running(&mut self) -> Result<bool, Self::Error> {
        Ok(self.try_wait()?.is_none())
    }
}

/// Process creation returns an owned child rather than a one-shot execution
/// result. Provider/session semantics remain above this boundary.
pub trait ProcessHost: Send + Sync {
    type Error: Error + Send + Sync + 'static;
    type Child: ProcessChild<Error = Self::Error>;

    fn spawn(
        &self,
        authority: &ProcessAuthority,
        request: ProcessRequest,
    ) -> Result<Self::Child, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use audiacore_errors::CodedError;

    fn absolute_root() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(r"C:\audiacore-test")
        } else {
            PathBuf::from("/audiacore-test")
        }
    }

    fn absolute_program() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(r"C:\audiacore-test\tool.exe")
        } else {
            PathBuf::from("/audiacore-test/tool")
        }
    }

    #[test]
    fn read_and_write_authorities_keep_their_grants_distinct() {
        let root = absolute_root();
        let read = FileReadAuthority::new(root.clone()).unwrap();
        let write = FileWriteAuthority::new(root.clone()).unwrap();

        assert_eq!(read.root(), root.as_path());
        assert_eq!(write.root(), root.as_path());
    }

    #[test]
    fn relative_file_roots_are_rejected_with_stable_identity() {
        let error = FileReadAuthority::new("relative/root").unwrap_err();
        assert_eq!(error.code().as_str(), "VAL-HOST-FILE-001");
    }

    #[test]
    fn process_authority_is_an_absolute_allow_list() {
        let program = absolute_program();
        let authority = ProcessAuthority::new([program.clone(), program.clone()]).unwrap();
        assert_eq!(authority.programs().len(), 1);
        assert!(authority.programs().contains(&program));

        let error = ProcessAuthority::new([PathBuf::from("relative/tool")]).unwrap_err();
        assert_eq!(error.code().as_str(), "VAL-HOST-PROCESS-001");
    }

    #[test]
    fn process_request_defaults_are_explicit_and_secret_safe() {
        let request = ProcessRequest::new(absolute_program())
            .unwrap()
            .env_secret(
                "TOKEN",
                Sensitive::new(OsString::from("never-log-this-value")),
            )
            .unwrap();

        assert!(!request.inherits_environment());
        assert_eq!(request.stdin_mode(), ProcessStdio::Pipe);
        assert_eq!(request.stdout_mode(), ProcessStdio::Pipe);
        assert_eq!(request.stderr_mode(), ProcessStdio::Pipe);

        let debug = format!("{request:?}");
        assert!(debug.contains("TOKEN"));
        assert!(!debug.contains("never-log-this-value"));
    }

    #[test]
    fn process_request_rejects_ambient_relative_paths_and_empty_env_keys() {
        let relative_program = ProcessRequest::new("relative/tool").unwrap_err();
        assert_eq!(relative_program.code().as_str(), "VAL-HOST-PROCESS-001");

        let relative_dir = ProcessRequest::new(absolute_program())
            .unwrap()
            .current_dir_path("relative/work")
            .unwrap_err();
        assert_eq!(relative_dir.code().as_str(), "VAL-HOST-PROCESS-002");

        let empty_key = ProcessRequest::new(absolute_program())
            .unwrap()
            .env_secret("", Sensitive::new(OsString::from("value")))
            .unwrap_err();
        assert_eq!(empty_key.code().as_str(), "VAL-HOST-PROCESS-003");
    }
}
