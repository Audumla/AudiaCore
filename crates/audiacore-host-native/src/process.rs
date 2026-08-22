use std::{
    error::Error,
    fmt,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio},
};

use audiacore_host::{
    ProcessAuthority, ProcessChild, ProcessExit, ProcessHost, ProcessRequest, ProcessStdio,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct NativeProcessHost;

#[derive(Debug)]
pub enum NativeProcessError {
    CanonicalizeProgram { path: PathBuf, source: io::Error },
    ProgramNotAuthorized(PathBuf),
    CanonicalizeCurrentDir { path: PathBuf, source: io::Error },
    CurrentDirNotDirectory(PathBuf),
    Spawn { program: PathBuf, source: io::Error },
    Inspect { pid: u32, source: io::Error },
    Wait { pid: u32, source: io::Error },
    Kill { pid: u32, source: io::Error },
}

impl fmt::Display for NativeProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CanonicalizeProgram { path, .. } => {
                write!(f, "cannot canonicalize process program {path:?}")
            }
            Self::ProgramNotAuthorized(path) => {
                write!(f, "process program is not authorized: {path:?}")
            }
            Self::CanonicalizeCurrentDir { path, .. } => {
                write!(f, "cannot canonicalize process working directory {path:?}")
            }
            Self::CurrentDirNotDirectory(path) => {
                write!(f, "process working directory is not a directory: {path:?}")
            }
            Self::Spawn { program, .. } => write!(f, "cannot spawn process {program:?}"),
            Self::Inspect { pid, .. } => write!(f, "cannot inspect process {pid}"),
            Self::Wait { pid, .. } => write!(f, "cannot wait for process {pid}"),
            Self::Kill { pid, .. } => write!(f, "cannot terminate process {pid}"),
        }
    }
}

impl Error for NativeProcessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalizeProgram { source, .. }
            | Self::CanonicalizeCurrentDir { source, .. }
            | Self::Spawn { source, .. }
            | Self::Inspect { source, .. }
            | Self::Wait { source, .. }
            | Self::Kill { source, .. } => Some(source),
            Self::ProgramNotAuthorized(_) | Self::CurrentDirNotDirectory(_) => None,
        }
    }
}

fn canonical_program(path: &Path) -> Result<PathBuf, NativeProcessError> {
    fs::canonicalize(path).map_err(|source| NativeProcessError::CanonicalizeProgram {
        path: path.to_path_buf(),
        source,
    })
}

fn authorize_program(
    authority: &ProcessAuthority,
    requested: &Path,
) -> Result<PathBuf, NativeProcessError> {
    let requested = canonical_program(requested)?;
    let authorized = authority.programs().iter().any(|allowed| {
        fs::canonicalize(allowed)
            .map(|canonical| canonical == requested)
            .unwrap_or(false)
    });

    if authorized {
        Ok(requested)
    } else {
        Err(NativeProcessError::ProgramNotAuthorized(requested))
    }
}

fn canonical_current_dir(path: &Path) -> Result<PathBuf, NativeProcessError> {
    let canonical =
        fs::canonicalize(path).map_err(|source| NativeProcessError::CanonicalizeCurrentDir {
            path: path.to_path_buf(),
            source,
        })?;
    if canonical.is_dir() {
        Ok(canonical)
    } else {
        Err(NativeProcessError::CurrentDirNotDirectory(canonical))
    }
}

fn native_stdio(mode: ProcessStdio) -> Stdio {
    match mode {
        ProcessStdio::Pipe => Stdio::piped(),
        ProcessStdio::Null => Stdio::null(),
        ProcessStdio::Inherit => Stdio::inherit(),
    }
}

fn process_exit(status: ExitStatus) -> ProcessExit {
    ProcessExit::new(status.code(), status.success())
}

pub struct NativeProcess {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
}

impl NativeProcess {
    fn from_child(mut child: Child) -> Self {
        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        Self {
            child,
            stdin,
            stdout,
            stderr,
        }
    }
}

impl ProcessChild for NativeProcess {
    type Error = NativeProcessError;

    fn id(&self) -> u32 {
        self.child.id()
    }

    fn take_stdin(&mut self) -> Option<Box<dyn Write + Send>> {
        self.stdin
            .take()
            .map(|stdin| Box::new(stdin) as Box<dyn Write + Send>)
    }

    fn take_stdout(&mut self) -> Option<Box<dyn Read + Send>> {
        self.stdout
            .take()
            .map(|stdout| Box::new(stdout) as Box<dyn Read + Send>)
    }

    fn take_stderr(&mut self) -> Option<Box<dyn Read + Send>> {
        self.stderr
            .take()
            .map(|stderr| Box::new(stderr) as Box<dyn Read + Send>)
    }

    fn try_wait(&mut self) -> Result<Option<ProcessExit>, Self::Error> {
        let pid = self.id();
        self.child
            .try_wait()
            .map(|status| status.map(process_exit))
            .map_err(|source| NativeProcessError::Inspect { pid, source })
    }

    fn wait(&mut self) -> Result<ProcessExit, Self::Error> {
        let pid = self.id();
        self.child
            .wait()
            .map(process_exit)
            .map_err(|source| NativeProcessError::Wait { pid, source })
    }

    fn kill(&mut self) -> Result<(), Self::Error> {
        let pid = self.id();
        if self.try_wait()?.is_some() {
            return Ok(());
        }
        self.child
            .kill()
            .map_err(|source| NativeProcessError::Kill { pid, source })
    }
}

impl Drop for NativeProcess {
    fn drop(&mut self) {
        match self.child.try_wait() {
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => {
                let _ = self.child.kill();
                let _ = self.child.wait();
            }
        }
    }
}

impl ProcessHost for NativeProcessHost {
    type Error = NativeProcessError;
    type Child = NativeProcess;

    fn spawn(
        &self,
        authority: &ProcessAuthority,
        request: ProcessRequest,
    ) -> Result<Self::Child, Self::Error> {
        let program = authorize_program(authority, request.program())?;
        let current_dir = request
            .current_dir()
            .map(canonical_current_dir)
            .transpose()?;

        let mut command = Command::new(&program);
        command.args(request.args());
        command.stdin(native_stdio(request.stdin_mode()));
        command.stdout(native_stdio(request.stdout_mode()));
        command.stderr(native_stdio(request.stderr_mode()));

        if let Some(current_dir) = current_dir {
            command.current_dir(current_dir);
        }
        if !request.inherits_environment() {
            command.env_clear();
        }
        for (key, value) in request.environment() {
            command.env(key, value.expose());
        }

        let child = command
            .spawn()
            .map_err(|source| NativeProcessError::Spawn {
                program: program.clone(),
                source,
            })?;
        Ok(NativeProcess::from_child(child))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use audiacore_sensitive::Sensitive;
    use std::{
        ffi::OsString,
        io::Read,
        thread,
        time::Duration,
    };

    const PROBE_MODE: &str = "AUDIACORE_NATIVE_PROCESS_PROBE";
    const PROBE_VALUE: &str = "AUDIACORE_NATIVE_PROCESS_VALUE";

    fn self_program() -> PathBuf {
        std::env::current_exe().unwrap()
    }

    fn self_authority() -> ProcessAuthority {
        ProcessAuthority::new([self_program()]).unwrap()
    }

    fn probe_request(mode: &str) -> ProcessRequest {
        ProcessRequest::new(self_program())
            .unwrap()
            .arg("native_process_probe")
            .arg("--nocapture")
            .stdin(ProcessStdio::Null)
            .env_secret(PROBE_MODE, Sensitive::new(OsString::from(mode)))
            .unwrap()
    }

    #[test]
    fn native_process_probe() {
        let Some(mode) = std::env::var_os(PROBE_MODE) else {
            return;
        };

        match mode.to_string_lossy().as_ref() {
            "environment" => {
                let value = std::env::var_os(PROBE_VALUE).unwrap_or_default();
                println!("AUDIACORE_PROBE_VALUE={}", value.to_string_lossy());
                println!(
                    "AUDIACORE_PROBE_PATH_PRESENT={}",
                    std::env::var_os("PATH").is_some()
                );
            }
            "sleep" => thread::sleep(Duration::from_secs(30)),
            other => panic!("unknown native process probe mode: {other}"),
        }
    }

    #[test]
    fn unauthorized_program_is_rejected_before_spawn() {
        let program = self_program();
        let authority = ProcessAuthority::new(std::iter::empty()).unwrap();
        let request = ProcessRequest::new(program).unwrap();

        let error = NativeProcessHost.spawn(&authority, request).unwrap_err();
        assert!(matches!(
            error,
            NativeProcessError::ProgramNotAuthorized(_)
        ));
    }

    #[test]
    fn explicit_environment_and_piped_output_work_without_ambient_path() {
        let request = probe_request("environment")
            .env_secret(
                PROBE_VALUE,
                Sensitive::new(OsString::from("explicit-value")),
            )
            .unwrap();
        let mut child = NativeProcessHost
            .spawn(&self_authority(), request)
            .unwrap();
        let mut stdout = child.take_stdout().unwrap();
        let mut output = String::new();
        stdout.read_to_string(&mut output).unwrap();
        let exit = child.wait().unwrap();

        assert!(exit.success(), "probe failed with output: {output}");
        assert!(output.contains("AUDIACORE_PROBE_VALUE=explicit-value"));
        assert!(output.contains("AUDIACORE_PROBE_PATH_PRESENT=false"));
    }

    #[test]
    fn live_direct_child_can_be_observed_killed_and_reaped() {
        let request = probe_request("sleep")
            .stdout(ProcessStdio::Null)
            .stderr(ProcessStdio::Null);
        let mut child = NativeProcessHost
            .spawn(&self_authority(), request)
            .unwrap();

        assert!(child.try_wait().unwrap().is_none());
        child.kill().unwrap();
        let exit = child.wait().unwrap();
        assert!(!exit.success());
    }
}
