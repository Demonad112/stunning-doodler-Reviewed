use rayon::prelude::*;
use serde::Serialize;
use std::collections::VecDeque;
use std::fmt;
use std::fs;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Index of a node in a [`Tree`]. The root is always 0.
pub type NodeId = u32;

/// Shared cancel flag. Clones point at the same flag.
#[derive(Clone, Debug, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// Live counters for one scan. The scan updates them; another thread reads [`ScanState::progress`].
#[derive(Debug, Default)]
pub struct ScanState {
    cancel: CancelToken,
    files: AtomicU64,
    dirs: AtomicU64,
    bytes: AtomicU64,
    current: Mutex<String>,
}

impl ScanState {
    pub fn new(cancel: CancelToken) -> Self {
        Self {
            cancel,
            ..Self::default()
        }
    }

    pub fn progress(&self) -> ScanProgress {
        ScanProgress {
            files: self.files.load(Ordering::Relaxed),
            dirs: self.dirs.load(Ordering::Relaxed),
            bytes: self.bytes.load(Ordering::Relaxed),
            current: self
                .current
                .lock()
                .map(|current| current.clone())
                .unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub files: u64,
    pub dirs: u64,
    pub bytes: u64,
    /// The folder being read most recently.
    pub current: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    File,
    Dir,
    /// A symlink or junction. Never followed, counted as size 0.
    Link,
}

/// One file, folder or link. Folder totals include everything below them.
#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
    pub kind: Kind,
    /// Logical size in bytes (what Explorer shows as "Size").
    pub size: u64,
    /// Files at or below this node (1 for a file).
    pub files: u64,
    /// Folders below this node, not counting itself.
    pub dirs: u64,
    /// Cloud-only placeholders (OneDrive "online-only") at or below this node. Their size is the
    /// logical size; the data is not on the disk.
    pub cloud_files: u64,
    /// Folders at or below this node that could not be read completely.
    pub errors: u64,
    /// Why this folder could not be read completely.
    pub error: Option<String>,
    /// The Windows error code behind [`Node::error`], when there is one.
    pub error_code: Option<i32>,
    /// Files: last modified time, in ms since 1970.
    pub modified_ms: Option<u64>,
    first_child: NodeId,
    child_count: u32,
}

impl Node {
    fn new(name: String, kind: Kind) -> Self {
        Self {
            name,
            kind,
            size: 0,
            files: 0,
            dirs: 0,
            cloud_files: 0,
            errors: 0,
            error: None,
            error_code: None,
            modified_ms: None,
            first_child: 0,
            child_count: 0,
        }
    }

    /// Ids of this node's children, largest first.
    pub fn children(&self) -> Range<NodeId> {
        self.first_child..self.first_child + self.child_count
    }

    pub fn has_children(&self) -> bool {
        self.child_count > 0
    }
}

/// A scanned folder as a flat arena. Children of a node are contiguous and sorted largest first.
#[derive(Clone, Debug)]
pub struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    pub fn root(&self) -> &Node {
        &self.nodes[0]
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id as usize]
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The child of `id` with exactly this name.
    pub fn child_by_name(&self, id: NodeId, name: &str) -> Option<NodeId> {
        self.node(id)
            .children()
            .find(|&child| self.node(child).name == name)
    }
}

#[derive(Debug)]
pub enum ScanError {
    NotFound(PathBuf),
    NotADirectory(PathBuf),
    Unreadable { path: PathBuf, message: String },
    Cancelled,
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "{} does not exist.", path.display()),
            Self::NotADirectory(path) => write!(f, "{} is not a folder.", path.display()),
            Self::Unreadable { path, message } => {
                write!(f, "{} could not be read: {message}", path.display())
            }
            Self::Cancelled => write!(f, "The scan was cancelled."),
        }
    }
}

impl std::error::Error for ScanError {}

/// Walks `root` and returns its size tree. Unreadable subfolders are kept with an error and the
/// scan continues; an unreadable root, a missing root or a cancel is an error.
pub fn scan(root: &Path, state: &ScanState) -> Result<Tree, ScanError> {
    let metadata = fs::metadata(root).map_err(|err| match err.kind() {
        io::ErrorKind::NotFound => ScanError::NotFound(root.to_path_buf()),
        _ => ScanError::Unreadable {
            path: root.to_path_buf(),
            message: err.to_string(),
        },
    })?;
    if !metadata.is_dir() {
        return Err(ScanError::NotADirectory(root.to_path_buf()));
    }

    let tree = scan_dir(root, root.display().to_string(), state);
    if state.cancel.is_cancelled() {
        return Err(ScanError::Cancelled);
    }
    if tree.node.error.is_some() && tree.children.is_empty() {
        return Err(ScanError::Unreadable {
            path: root.to_path_buf(),
            message: tree.node.error.unwrap_or_default(),
        });
    }
    Ok(flatten(tree))
}

/// A node with owned children, built during the walk and then flattened into a [`Tree`].
struct Owned {
    node: Node,
    children: Vec<Owned>,
}

impl Owned {
    fn leaf(name: String, kind: Kind, size: u64, cloud: bool) -> Self {
        let mut node = Node::new(name, kind);
        node.size = size;
        node.files = u64::from(kind == Kind::File);
        node.cloud_files = u64::from(cloud);
        Self {
            node,
            children: Vec::new(),
        }
    }
}

fn scan_dir(path: &Path, name: String, state: &ScanState) -> Owned {
    let mut dir = Owned {
        node: Node::new(name, Kind::Dir),
        children: Vec::new(),
    };
    if state.cancel.is_cancelled() {
        return dir;
    }
    state.dirs.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut current) = state.current.try_lock() {
        *current = path.display().to_string();
    }

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) => {
            dir.node.error = Some(err.to_string());
            dir.node.error_code = err.raw_os_error();
            dir.node.errors = 1;
            return dir;
        }
    };

    let mut subdirs = Vec::new();
    let mut files = 0;
    let mut bytes = 0;
    for entry in entries {
        // On Windows the entry's type and metadata come from the directory listing itself
        // (FindNextFileW), so this costs no extra system call per file.
        let entry_info = entry.and_then(|entry| {
            let file_type = entry.file_type()?;
            Ok((entry, file_type))
        });
        let (entry, file_type) = match entry_info {
            Ok(info) => info,
            Err(err) => {
                dir.node.error.get_or_insert_with(|| err.to_string());
                continue;
            }
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        if file_type.is_symlink() {
            dir.children.push(Owned::leaf(name, Kind::Link, 0, false));
        } else if file_type.is_dir() {
            subdirs.push((entry.path(), name));
        } else {
            match entry.metadata() {
                Ok(metadata) => {
                    files += 1;
                    bytes += metadata.len();
                    let cloud = is_cloud_placeholder(&metadata);
                    let mut leaf = Owned::leaf(name, Kind::File, metadata.len(), cloud);
                    leaf.node.modified_ms = metadata
                        .modified()
                        .ok()
                        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|since| u64::try_from(since.as_millis()).unwrap_or(u64::MAX));
                    dir.children.push(leaf);
                }
                Err(err) => {
                    dir.node.error.get_or_insert_with(|| err.to_string());
                }
            }
        }
    }
    state.files.fetch_add(files, Ordering::Relaxed);
    state.bytes.fetch_add(bytes, Ordering::Relaxed);

    let subdirs: Vec<Owned> = subdirs
        .into_par_iter()
        .map(|(path, name)| scan_dir(&path, name, state))
        .collect();
    dir.children.extend(subdirs);

    let node = &mut dir.node;
    node.errors = u64::from(node.error.is_some());
    for child in &dir.children {
        node.size += child.node.size;
        node.files += child.node.files;
        node.dirs += child.node.dirs + u64::from(child.node.kind == Kind::Dir);
        node.cloud_files += child.node.cloud_files;
        node.errors += child.node.errors;
    }
    dir.children.sort_by(|a, b| {
        b.node
            .size
            .cmp(&a.node.size)
            .then_with(|| a.node.name.cmp(&b.node.name))
    });
    dir
}

/// Breadth-first, so every node's children end up next to each other.
fn flatten(root: Owned) -> Tree {
    let mut nodes = vec![root.node];
    let mut queue = VecDeque::from([(0usize, root.children)]);
    while let Some((id, children)) = queue.pop_front() {
        nodes[id].first_child = nodes.len() as NodeId;
        nodes[id].child_count = children.len() as u32;
        for child in children {
            nodes.push(child.node);
            queue.push_back((nodes.len() - 1, child.children));
        }
    }
    Tree { nodes }
}

/// OneDrive and other cloud providers mark online-only files with these attributes. Reading the
/// metadata never downloads them.
#[cfg(windows)]
fn is_cloud_placeholder(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_OFFLINE: u32 = 0x1000;
    const FILE_ATTRIBUTE_RECALL_ON_OPEN: u32 = 0x4_0000;
    const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x40_0000;
    metadata.file_attributes()
        & (FILE_ATTRIBUTE_OFFLINE
            | FILE_ATTRIBUTE_RECALL_ON_OPEN
            | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS)
        != 0
}

#[cfg(not(windows))]
fn is_cloud_placeholder(_metadata: &fs::Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(path: &Path, bytes: usize) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, vec![b'x'; bytes]).unwrap();
    }

    fn scan_ok(root: &Path) -> Tree {
        scan(root, &ScanState::default()).unwrap()
    }

    #[test]
    fn folder_totals_are_recursive() {
        let temp = tempfile::tempdir().unwrap();
        write(&temp.path().join("a.bin"), 10);
        write(&temp.path().join("sub/b.bin"), 20);
        write(&temp.path().join("sub/deeper/c.bin"), 30);
        fs::create_dir(temp.path().join("empty")).unwrap();

        let tree = scan_ok(temp.path());
        let root = tree.root();
        let file = tree.child_by_name(0, "a.bin").unwrap();
        assert!(tree.node(file).modified_ms.is_some());
        assert_eq!(root.size, 60);
        assert_eq!(root.files, 3);
        assert_eq!(root.dirs, 3);

        let sub = tree.child_by_name(0, "sub").unwrap();
        assert_eq!(tree.node(sub).size, 50);
        assert_eq!(tree.node(sub).files, 2);
        assert_eq!(tree.node(sub).dirs, 1);

        let empty = tree.child_by_name(0, "empty").unwrap();
        assert_eq!(tree.node(empty).size, 0);
        assert!(!tree.node(empty).has_children());
    }

    #[test]
    fn children_are_contiguous_and_largest_first() {
        let temp = tempfile::tempdir().unwrap();
        write(&temp.path().join("small.bin"), 1);
        write(&temp.path().join("big.bin"), 100);
        write(&temp.path().join("mid/x.bin"), 50);

        let tree = scan_ok(temp.path());
        let names: Vec<&str> = tree
            .root()
            .children()
            .map(|id| tree.node(id).name.as_str())
            .collect();
        assert_eq!(names, ["big.bin", "mid", "small.bin"]);
        assert_eq!(tree.len(), 5);
    }

    #[test]
    fn progress_counts_everything() {
        let temp = tempfile::tempdir().unwrap();
        write(&temp.path().join("a.bin"), 5);
        write(&temp.path().join("sub/b.bin"), 7);
        let state = ScanState::default();
        scan(temp.path(), &state).unwrap();
        let progress = state.progress();
        assert_eq!(progress.files, 2);
        assert_eq!(progress.bytes, 12);
        assert_eq!(progress.dirs, 2);
    }

    #[test]
    fn missing_root_and_file_root_are_errors() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("file.txt");
        write(&file, 1);
        assert!(matches!(
            scan(&temp.path().join("nope"), &ScanState::default()),
            Err(ScanError::NotFound(_))
        ));
        assert!(matches!(
            scan(&file, &ScanState::default()),
            Err(ScanError::NotADirectory(_))
        ));
    }

    #[test]
    fn cancelled_scan_is_an_error() {
        let temp = tempfile::tempdir().unwrap();
        write(&temp.path().join("a.bin"), 5);
        let cancel = CancelToken::new();
        cancel.cancel();
        assert!(matches!(
            scan(temp.path(), &ScanState::new(cancel)),
            Err(ScanError::Cancelled)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_listed_but_not_followed() {
        let temp = tempfile::tempdir().unwrap();
        write(&temp.path().join("real/data.bin"), 40);
        std::os::unix::fs::symlink(temp.path().join("real"), temp.path().join("link")).unwrap();
        // A loop back to the root must not hang the scan.
        std::os::unix::fs::symlink(temp.path(), temp.path().join("real/loop")).unwrap();

        let tree = scan_ok(temp.path());
        assert_eq!(tree.root().size, 40);
        assert_eq!(tree.root().files, 1);
        let link = tree.child_by_name(0, "link").unwrap();
        assert_eq!(tree.node(link).kind, Kind::Link);
        assert_eq!(tree.node(link).size, 0);
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_subfolder_is_kept_with_an_error() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        write(&temp.path().join("ok.bin"), 3);
        let locked = temp.path().join("locked");
        write(&locked.join("secret.bin"), 9);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
        // Root ignores permissions, so the folder stays readable there; skip in that case.
        let readable = fs::read_dir(&locked).is_ok();

        let tree = scan_ok(temp.path());
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        if readable {
            return;
        }
        let id = tree.child_by_name(0, "locked").unwrap();
        assert!(tree.node(id).error.is_some());
        assert_eq!(tree.root().errors, 1);
        assert_eq!(tree.root().size, 3);
    }
}
