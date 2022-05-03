use elysh_theme::Style;
use fst::automaton::Str;
use fst::{Automaton, IntoStreamer, Set};
use std::collections::{BTreeMap, HashSet};
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::{env, fmt, hint};
use tokio::{fs, io};

/// A map of executables.
pub struct Exes {
    set: Set<Vec<u8>>,
}

impl Exes {
    /// Construct a new map of executables.
    #[inline]
    pub fn new(set: &BTreeMap<String, PathBuf>) -> Self {
        unsafe {
            let iter = set.keys();
            let set = Set::from_iter(iter).unwrap_unchecked();

            Self { set }
        }
    }

    /// Construct a new map of executables from env.
    #[inline]
    pub async fn from_env() -> io::Result<Self> {
        let exes = from_env().await?;

        Ok(Self::new(&exes))
    }

    /// Search for executables by the provided query.
    #[inline]
    fn fst_search(&self, query: &str) -> Vec<String> {
        let query = Str::new(query).starts_with();
        let stream = self.set.search(query).into_stream();

        stream.into_strs().unwrap_or_default()
    }

    /// Search for executables by the provided query.
    #[inline]
    pub fn search(&self, query: &str) -> Vec<Summary> {
        let results = self.fst_search(query);

        results
            .into_iter()
            .map(|result| match result.strip_prefix(query) {
                Some(rest) => {
                    if rest.is_empty() {
                        Summary::Exact(Box::from(query))
                    } else {
                        Summary::Partial(Box::from(query), Box::from(rest))
                    }
                }
                None => unsafe { hint::unreachable_unchecked() },
            })
            .collect()
    }

    /// Search for an executable by the provided query.
    #[inline]
    pub fn search_one(&self, query: &str) -> Summary {
        match self.search(query).into_iter().next() {
            Some(result) => result,
            None => Summary::NoMatch,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Summary {
    Exact(Box<str>),
    Partial(Box<str>, Box<str>),
    NoMatch,
}

impl Summary {
    #[inline]
    pub const fn is_exact(&self) -> bool {
        matches!(self, Summary::Exact(_))
    }

    #[inline]
    pub const fn is_partial(&self) -> bool {
        matches!(self, Summary::Partial(_, _))
    }

    #[inline]
    pub const fn is_no_match(&self) -> bool {
        matches!(self, Summary::NoMatch)
    }

    #[inline]
    pub const fn shift(&self) -> usize {
        match self {
            Summary::Partial(_partial, rest) => rest.len(),
            _ => 0,
        }
    }

    #[inline]
    pub fn display<'a>(&'a self, exact: &'a Style, rest: &'a Style) -> Option<SummaryDisplay<'a>> {
        if self.is_no_match() {
            return None;
        } else {
            Some(SummaryDisplay {
                exact,
                rest,
                summary: &self,
            })
        }
    }
}

pub struct SummaryDisplay<'a> {
    exact: &'a Style,
    rest: &'a Style,
    summary: &'a Summary,
}

impl<'a> fmt::Display for SummaryDisplay<'a> {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.summary {
            Summary::Exact(exact) => {
                fmt.write_str(self.exact.as_ansi())?;
                fmt.write_str(exact)?;
                fmt.write_str("\x1b[m")?;
            }
            Summary::Partial(partial, rest) => {
                fmt.write_str(partial)?;
                fmt.write_str(self.rest.as_ansi())?;
                fmt.write_str(rest)?;
                fmt.write_str("\x1b[m")?;
            }
            _ => unsafe { hint::unreachable_unchecked() },
        }

        Ok(())
    }
}

/// Checks the provided metadata if it is executable for the provided user and group.
#[inline]
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
#[inline]
pub async fn iterate_dir(bins: &mut HashSet<PathBuf>, path: PathBuf) -> io::Result<()> {
    let mut entries = fs::read_dir(path).await?;

    while let Some(entry) = entries.next_entry().await? {
        bins.insert(entry.path());
    }

    Ok(())
}

/// Collects a map of executables from the `PATH` envrionment variable.
#[inline]
async fn from_env() -> io::Result<BTreeMap<String, PathBuf>> {
    let path = env::var_os("PATH").unwrap_or_default();
    let paths = env::split_paths(&path);
    let user = unsafe { cream::env::current_user().unwrap_unchecked() };
    let group = unsafe { cream::env::current_group().unwrap_unchecked() };

    let set = {
        let mut set = HashSet::new();

        for path in paths {
            let _ = iterate_dir(&mut set, path).await;
        }

        set
    };

    let bins = {
        let mut bins = BTreeMap::new();

        bins.insert("cd".into(), "<builtin>".into());
        bins.insert("showkeys".into(), "<builtin>".into());

        for bin in set {
            let metadata = match fs::metadata(&bin).await {
                Ok(metadata) => metadata,
                Err(_error) => continue,
            };

            if !(metadata.is_file() && can_execute(user, group, &metadata)) {
                continue;
            }

            let name = match bin.file_name() {
                Some(name) => name.to_string_lossy().into(),
                None => continue,
            };

            bins.insert(name, bin);
        }

        bins
    };

    Ok(bins)
}
