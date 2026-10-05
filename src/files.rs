use crate::*;

pub(crate) fn list_files(root: &Path, limit: usize) -> FileScanSnapshot {
    visit(root, limit, |path| path.is_file())
}

/// Fold the flat list of paths a scan returns back into the tree they came
/// from. Directories are the ones the paths imply, so an empty one — or one
/// holding nothing the scan accepted — simply is not here.
///
/// At each level directories come first and then files, both by name, which is
/// the order every file tree a person has used puts them in.
pub(crate) fn build_file_tree(paths: &[PathBuf]) -> Vec<FileTreeNode> {
    fn insert(level: &mut Vec<FileTreeNode>, prefix: &Path, mut components: std::path::Components) {
        let Some(component) = components.next() else {
            return;
        };
        let name = component.as_os_str().to_string_lossy().into_owned();
        let path = prefix.join(component);
        // The scan hands paths over sorted, so the node this path needs is
        // almost always the one just added. Checking it first is what keeps a
        // directory of ten thousand files from costing ten thousand scans of
        // its own siblings.
        let index = if level.last().is_some_and(|node| node.name == name) {
            level.len() - 1
        } else if let Some(index) = level.iter().position(|node| node.name == name) {
            index
        } else {
            level.push(FileTreeNode {
                name,
                path: path.clone(),
                children: Vec::new(),
            });
            level.len() - 1
        };
        insert(&mut level[index].children, &path, components);
    }

    fn sort(level: &mut Vec<FileTreeNode>) {
        level.sort_by(|left, right| {
            right
                .is_directory()
                .cmp(&left.is_directory())
                .then_with(|| left.name.cmp(&right.name))
        });
        for node in level {
            sort(&mut node.children);
        }
    }

    let mut roots = Vec::new();
    for path in paths {
        insert(&mut roots, Path::new(""), path.components());
    }
    sort(&mut roots);
    roots
}

/// A common project-text format the editor can colour without needing to read
/// the whole tree. The extension is only a display hint: all files still open
/// as text when they pass the editor's safety checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum EditorLanguage {
    Plain,
    Markdown,
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Ruby,
    Shell,
    C,
    Cpp,
    Swift,
    Kotlin,
    Java,
    CSharp,
    ObjectiveC,
    Php,
    Vue,
    Sql,
    Html,
    Xml,
    Css,
    Json,
    Toml,
    Yaml,
    Ini,
    Dockerfile,
    Makefile,
    Lua,
}

/// The one extension classification shared by the project-tree icon and the
/// editor. Keeping them together stops a format that looks like code in the
/// tree from unexpectedly opening as uncoloured prose.
pub(crate) fn editor_language(path: &Path) -> EditorLanguage {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if matches!(name.as_str(), "dockerfile" | "containerfile") {
        return EditorLanguage::Dockerfile;
    }
    if matches!(name.as_str(), "makefile" | "gnumakefile") {
        return EditorLanguage::Makefile;
    }
    if name == ".env" || name.starts_with(".env.") {
        return EditorLanguage::Shell;
    }
    match extension.as_str() {
        "md" | "markdown" | "mdx" => EditorLanguage::Markdown,
        "rs" => EditorLanguage::Rust,
        "js" | "jsx" | "mjs" | "cjs" => EditorLanguage::JavaScript,
        "ts" | "tsx" | "mts" | "cts" => EditorLanguage::TypeScript,
        "py" => EditorLanguage::Python,
        "go" => EditorLanguage::Go,
        "rb" => EditorLanguage::Ruby,
        "sh" | "bash" | "zsh" => EditorLanguage::Shell,
        "c" => EditorLanguage::C,
        "h" | "cc" | "cp" | "cpp" | "cxx" | "hpp" | "hxx" => EditorLanguage::Cpp,
        "swift" => EditorLanguage::Swift,
        "kt" | "kts" => EditorLanguage::Kotlin,
        "java" => EditorLanguage::Java,
        "cs" => EditorLanguage::CSharp,
        "m" | "mm" => EditorLanguage::ObjectiveC,
        "php" => EditorLanguage::Php,
        "vue" => EditorLanguage::Vue,
        "sql" => EditorLanguage::Sql,
        "html" | "htm" => EditorLanguage::Html,
        "xml" | "xsd" | "xsl" | "xslt" | "svg" => EditorLanguage::Xml,
        "css" => EditorLanguage::Css,
        "json" => EditorLanguage::Json,
        "toml" => EditorLanguage::Toml,
        "yaml" | "yml" => EditorLanguage::Yaml,
        "ini" | "cfg" | "conf" | "properties" => EditorLanguage::Ini,
        "lua" => EditorLanguage::Lua,
        _ => EditorLanguage::Plain,
    }
}

/// The language a fenced code block's info string names.
///
/// The project tree classifies by extension and a fence is labelled with a
/// word, so the words agents write that are not also extensions are listed
/// here, and everything else is tried as an extension through
/// `editor_language`. That is what makes a fence marked `rs`, `py`, `yml`, or
/// `json` work without a second table to keep in step with the first, and what
/// makes `text` and `output` land on `Plain` — which is the value
/// `syntax_layout` reads as "leave this alone".
///
/// Only the first word counts. A fence may carry attributes after its language
/// (`rust,ignore`, `python title="x"`), and the language is the part before
/// them.
pub(crate) fn fence_language(tag: &str) -> EditorLanguage {
    let word = tag
        .split([' ', '\t', ',', ';', '{'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match word.as_str() {
        "" => EditorLanguage::Plain,
        "rust" => EditorLanguage::Rust,
        "javascript" | "node" => EditorLanguage::JavaScript,
        "typescript" => EditorLanguage::TypeScript,
        "python" => EditorLanguage::Python,
        "golang" => EditorLanguage::Go,
        "ruby" => EditorLanguage::Ruby,
        "shell" | "console" | "terminal" | "shell-session" => EditorLanguage::Shell,
        "objective-c" | "objc" => EditorLanguage::ObjectiveC,
        "c++" => EditorLanguage::Cpp,
        "c#" | "csharp" => EditorLanguage::CSharp,
        "kotlin" => EditorLanguage::Kotlin,
        "dockerfile" => EditorLanguage::Dockerfile,
        "makefile" | "make" => EditorLanguage::Makefile,
        other => editor_language(Path::new(&format!("fence.{other}"))),
    }
}

/// Which of the three file faces a name gets. Extension only: reading the file
/// to find out would mean reading every file in the tree.
pub(crate) fn file_tree_icon(name: &str) -> &'static str {
    match editor_language(Path::new(name)) {
        EditorLanguage::Markdown => ICON_FILE_MARKDOWN,
        EditorLanguage::Plain => ICON_FILE,
        _ => ICON_FILE_CODE,
    }
}

/// Whether this file is Markdown, which is the one format the editor can show
/// two ways.
pub(crate) fn is_markdown(path: &Path) -> bool {
    editor_language(path) == EditorLanguage::Markdown
}

/// Whether this file is a supported image format (PNG, JPEG, GIF, WebP, BMP, ICO)
/// that can be visually previewed in the editor.
pub(crate) fn is_image(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico"
    )
}
pub(crate) fn file_scan_warning(scan: &FileScanSnapshot) -> Option<String> {
    (scan.truncated || scan.unreadable_directories > 0).then(|| {
        tf!(
            "上限付きのプロジェクトスキャン結果を表示しています。{truncated}{unreadable}",
            truncated = if scan.truncated {
                tr(" 一部のファイルは省略されています。フォルダで絞り込むか、外部エディタを使ってください。")
            } else {
                ""
            },
            unreadable = if scan.unreadable_directories > 0 {
                tf!(" {unreadable_directories} 件のディレクトリを読み取れませんでした。", unreadable_directories = scan.unreadable_directories)
            } else {
                String::new()
            }
        )
    })
}
pub(crate) fn find_named_files(root: &Path, name: &str, limit: usize) -> FileScanSnapshot {
    visit(root, limit, |path| {
        path.file_name().is_some_and(|file| file == name)
    })
}
pub(crate) fn find_named_files_any(root: &Path, names: &[&str], limit: usize) -> FileScanSnapshot {
    visit(root, limit, |path| {
        path.file_name()
            .and_then(|file| file.to_str())
            .is_some_and(|file| names.contains(&file))
    })
}
pub(crate) fn visit(
    root: &Path,
    limit: usize,
    accept: impl Fn(&Path) -> bool + Copy,
) -> FileScanSnapshot {
    visit_with_limits(
        root,
        limit,
        FILE_SCAN_VISIT_LIMIT,
        DIRECTORY_ENTRY_BUFFER_LIMIT,
        accept,
    )
}
pub(crate) fn visit_with_limits(
    root: &Path,
    limit: usize,
    visit_limit: usize,
    directory_entry_limit: usize,
    accept: impl Fn(&Path) -> bool + Copy,
) -> FileScanSnapshot {
    let mut scan = FileScanSnapshot::default();
    let mut directories = VecDeque::from([root.to_path_buf()]);
    let mut visited = 0;
    while let Some(current) = directories.pop_front() {
        if scan.paths.len() >= limit || visited >= visit_limit {
            scan.truncated = true;
            break;
        }
        let entries = match fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(_) => {
                scan.unreadable_directories += 1;
                continue;
            }
        };
        let entry_budget = directory_entry_limit.min(visit_limit.saturating_sub(visited));
        let mut entries = entries.take(entry_budget).collect::<Vec<_>>();
        // Conservatively disclose truncation when the per-directory buffer is
        // full. This keeps both allocation and sorting bounded even for a
        // directory with millions of entries.
        if entries.len() == entry_budget {
            scan.truncated = true;
        }
        entries.sort_by_key(|entry| {
            entry
                .as_ref()
                .map(|entry| entry.file_name())
                .unwrap_or_default()
        });
        for entry in entries {
            if scan.paths.len() >= limit || visited >= visit_limit {
                scan.truncated = true;
                break;
            }
            visited += 1;
            let Ok(entry) = entry else {
                scan.unreadable_directories += 1;
                continue;
            };
            let Ok(file_type) = entry.file_type() else {
                scan.unreadable_directories += 1;
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap_or(&path);
            if relative.components().any(|part| {
                matches!(
                    part.as_os_str().to_str(),
                    Some(".git" | "node_modules" | "target" | ".next")
                )
            }) {
                continue;
            }
            if accept(&path) {
                scan.paths.push(relative.to_path_buf());
            }
            if file_type.is_dir() {
                directories.push_back(path);
            }
        }
    }
    scan.paths.sort();
    scan
}
