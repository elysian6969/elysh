use fst::automaton::Str;
use fst::{Automaton, IntoStreamer, Set};
use std::collections::BTreeMap;
use std::env;
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use tokio::{fs, io};

/// A map of executables.
pub struct Executables {
    set: Set<Vec<u8>>,
}

impl Executables {
    /// Construct a new map of executables.
    pub fn new(set: &BTreeMap<String, PathBuf>) -> Self {
        unsafe {
            let iter = set.keys();
            let set = Set::from_iter(iter).unwrap_unchecked();

            Self { set }
        }
    }

    /// Search for executables by the provided query.
    pub fn search(&self, query: &str) -> Vec<String> {
        let query = Str::new(query).starts_with();
        let stream = self.set.search(query).into_stream();

        stream.into_strs().unwrap_or_default()
    }

    /// Search for an executable by the provided query.
    pub fn search_one(&self, query: &str) -> Option<String> {
        self.search(query).into_iter().next()
    }
}

/// Checks the provided metadata if it is executable for the provided user and group.
pub fn can_execute(user: u32, group: u32, metadata: &Metadata) -> bool {
    let mode = metadata.mode();
    let user = metadata.uid() == user;
    let group = metadata.gid() == group;

    let user_execute = mode & 0o100 != 0;
    let group_execute = mode & 0o010 != 0;
    let other_execute = mode & 0o001 != 0;

    (user && user_execute) || (group && group_execute) || other_execute
}

/// Iterate a directory.
pub async fn iterate_dir(
    executables: &mut BTreeMap<String, PathBuf>,
    user: u32,
    group: u32,
    path: PathBuf,
) -> io::Result<()> {
    let mut entries = fs::read_dir(path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;

        if !(metadata.is_file() && can_execute(user, group, &metadata)) {
            continue;
        }

        let executable = entry.file_name().to_string_lossy().to_string();
        let path = fs::canonicalize(entry.path()).await?;

        executables.insert(executable, path);
    }

    Ok(())
}

/// Collects a map of executables from the `PATH` envrionment variable.
pub async fn from_env() -> io::Result<BTreeMap<String, PathBuf>> {
    let path = env::var_os("PATH").unwrap_or_default();
    let paths = env::split_paths(&path);
    let user = unsafe { cream::env::current_user().unwrap_unchecked() };
    let group = unsafe { cream::env::current_group().unwrap_unchecked() };
    let mut executables = BTreeMap::new();

    executables.insert("cd".into(), "<builtin>".into());

    for path in paths {
        let _ = iterate_dir(&mut executables, user, group, path).await;
    }

    Ok(executables)
}
