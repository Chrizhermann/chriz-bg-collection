//! Unix compatibility normalizations applied only to staged mod files, never cached archives.

use std::collections::HashSet;
use std::path::Path;

use serde::Serialize;

#[cfg(unix)]
use crate::digest::sha256_bytes;
#[cfg(unix)]
use crate::held_folder::{EntryError, EntryErrorKind, HeldFolder};

#[cfg(unix)]
const MAX_MOD_TREE_DEPTH: usize = 16;
#[cfg(unix)]
const MAX_TP2_SCAN_BYTES: u64 = 16 * 1024 * 1024;

pub(crate) struct UnixModCompatibilityInput<'a> {
    #[cfg_attr(not(unix), allow(dead_code))]
    pub mod_id: &'a str,
    #[cfg_attr(not(unix), allow(dead_code))]
    pub tp2: &'a str,
    #[cfg_attr(not(unix), allow(dead_code))]
    pub target_root: &'a Path,
}

#[derive(Debug, Serialize)]
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) struct UnixModCompatibilityEvidence {
    fix_id: &'static str,
    mod_id: String,
    tp2: String,
    patched_files: Vec<PatchedModFile>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) struct PatchedModFile {
    relative_path: String,
    observed_before_sha256: String,
    observed_after_sha256: String,
}

/// Normalizes staged mod TP2/TPA/TPH/TPP files on Unix before WeiDU executes so that:
/// 1. `(ACTION|PATCH)_BASH_FOR` directory operands do not carry trailing slashes (which trigger
///    `Unix.Unix_error(Unix.ENOENT, "stat", "MOD/SUB//FILE")` or silent partial lists on WeiDU 251).
/// 2. Legacy `READ_2DA_ENTRIES_NOW <name> <cols>` calls paired with `READ_2DA_ENTRY_FORMER <name>`
///    and `(ACTION_)PHP_EACH <name> AS ...` clear the associative `$<name>("<row>", "<col>")`
///    entries populated by WeiDU 251 while preserving scalar row counts and `_row_col` variables.
pub(crate) fn prepare(
    input: UnixModCompatibilityInput<'_>,
) -> Result<Option<UnixModCompatibilityEvidence>, String> {
    #[cfg(not(unix))]
    {
        let _ = input;
        Ok(None)
    }
    #[cfg(unix)]
    {
        prepare_unix(input)
    }
}

#[cfg(unix)]
fn prepare_unix(
    input: UnixModCompatibilityInput<'_>,
) -> Result<Option<UnixModCompatibilityEvidence>, String> {
    if !input.target_root.is_absolute() {
        return Err("Unix mod compatibility target must be absolute".into());
    }
    let normalized_tp2 = input.tp2.replace('\\', "/");
    let trimmed_tp2 = normalized_tp2.trim_start_matches("./");
    let segments: Vec<&str> = trimmed_tp2
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.is_empty()
        || segments
            .iter()
            .any(|segment| *segment == "." || *segment == "..")
    {
        return Ok(None);
    }

    let root = HeldFolder::open(input.target_root, []).map_err(path_error)?;
    let mut patched_files = Vec::new();

    if segments.len() >= 2 {
        let folder = match root.open_below([segments[0]]) {
            Ok(folder) => folder,
            Err(error) if matches!(error.kind, EntryErrorKind::Missing) => return Ok(None),
            Err(error) => return Err(path_error(error)),
        };
        walk_mod_folder(&folder, input.target_root, 0, &mut patched_files)?;
    } else {
        let tp2_name = segments[0];
        inspect_top_level_file(&root, tp2_name, input.target_root, &mut patched_files)?;
        if let Some(companion) = companion_folder_name(tp2_name) {
            if companion != "." && companion != ".." {
                match root.open_below([companion]) {
                    Ok(folder) => {
                        walk_mod_folder(&folder, input.target_root, 0, &mut patched_files)?;
                    }
                    Err(error) if matches!(error.kind, EntryErrorKind::Missing) => {}
                    Err(error) => return Err(path_error(error)),
                }
            }
        }
    }

    if patched_files.is_empty() {
        return Ok(None);
    }
    Ok(Some(UnixModCompatibilityEvidence {
        fix_id: "weidu-unix-mod-compatibility-v1",
        mod_id: input.mod_id.to_owned(),
        tp2: input.tp2.to_owned(),
        patched_files,
    }))
}

#[cfg(unix)]
fn companion_folder_name(tp2_name: &str) -> Option<&str> {
    let suffix_start = tp2_name.len().checked_sub(4)?;
    if !tp2_name
        .as_bytes()
        .get(suffix_start..)
        .is_some_and(|ext| ext.eq_ignore_ascii_case(b".tp2"))
    {
        return None;
    }
    let stem = &tp2_name[..suffix_start];
    let trimmed = if stem
        .as_bytes()
        .get(..6)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"setup-"))
    {
        &stem[6..]
    } else {
        stem
    };
    (!trimmed.is_empty()).then_some(trimmed)
}

#[cfg(unix)]
fn inspect_top_level_file(
    root: &HeldFolder,
    name: &str,
    target_root: &Path,
    patched_files: &mut Vec<PatchedModFile>,
) -> Result<(), String> {
    if !is_weidu_script_filename(name) {
        return Ok(());
    }
    let (file, metadata) = match root.open_file(name) {
        Ok(opened) => opened,
        Err(error) if matches!(error.kind, EntryErrorKind::Missing) => return Ok(()),
        Err(error) => return Err(path_error(error)),
    };
    patch_opened_file(root, name, file, metadata, target_root, patched_files)
}

#[cfg(unix)]
fn walk_mod_folder(
    folder: &HeldFolder,
    target_root: &Path,
    depth: usize,
    patched_files: &mut Vec<PatchedModFile>,
) -> Result<(), String> {
    let mut entries = folder.entries().map_err(|error| {
        format!(
            "Inspect Unix mod compatibility directory {}: {error}",
            folder.path().display()
        )
    })?;
    entries.sort_by(|left, right| left.name.cmp(&right.name));

    for entry in entries {
        let name = entry.name.to_str().ok_or_else(|| {
            format!(
                "Non-UTF-8 entry inside Unix mod compatibility folder {}",
                folder.path().display()
            )
        })?;
        if entry.is_reparse_point() {
            return Err(format!(
                "Unix mod compatibility path is not a direct file/directory: {}",
                folder.path().join(name).display()
            ));
        }
        if entry.is_directory() {
            if depth == 0 && name.eq_ignore_ascii_case("backup") {
                continue;
            }
            if depth < MAX_MOD_TREE_DEPTH {
                let child = folder.open_below([name]).map_err(path_error)?;
                walk_mod_folder(&child, target_root, depth + 1, patched_files)?;
            }
            continue;
        }
        if !is_weidu_script_filename(name) {
            continue;
        }
        let (file, metadata) = match folder.open_file(name) {
            Ok(opened) => opened,
            Err(error) if matches!(error.kind, EntryErrorKind::Missing) => continue,
            Err(error) => return Err(path_error(error)),
        };
        patch_opened_file(folder, name, file, metadata, target_root, patched_files)?;
    }
    Ok(())
}

#[cfg(unix)]
fn patch_opened_file(
    folder: &HeldFolder,
    name: &str,
    file: std::fs::File,
    metadata: std::fs::Metadata,
    target_root: &Path,
    patched_files: &mut Vec<PatchedModFile>,
) -> Result<(), String> {
    use std::io::Read;

    if metadata.len() > MAX_TP2_SCAN_BYTES {
        return Ok(());
    }
    let path = folder.path().join(name);
    let mut before = Vec::with_capacity(metadata.len() as usize);
    (&file)
        .take(MAX_TP2_SCAN_BYTES + 1)
        .read_to_end(&mut before)
        .map_err(|error| {
            format!(
                "Read Unix mod compatibility file {}: {error}",
                path.display()
            )
        })?;
    if before.len() as u64 > MAX_TP2_SCAN_BYTES {
        return Ok(());
    }
    let Some(after) = rewrite_unix_mod_script(&before) else {
        return Ok(());
    };
    folder.replace_file(name, &after).map_err(|error| {
        format!(
            "Replace Unix mod compatibility file {}: {error}",
            path.display()
        )
    })?;
    let relative = path.strip_prefix(target_root).map_err(|error| {
        format!(
            "Compute relative path for Unix mod compatibility file {}: {error}",
            path.display()
        )
    })?;
    patched_files.push(PatchedModFile {
        relative_path: relative.to_string_lossy().replace('\\', "/"),
        observed_before_sha256: sha256_bytes(&before),
        observed_after_sha256: sha256_bytes(&after),
    });
    Ok(())
}

#[cfg(unix)]
fn is_weidu_script_filename(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(_, ext)| {
        ext.eq_ignore_ascii_case("tp2")
            || ext.eq_ignore_ascii_case("tpa")
            || ext.eq_ignore_ascii_case("tph")
            || ext.eq_ignore_ascii_case("tpp")
    })
}

#[cfg(unix)]
fn path_error(error: EntryError) -> String {
    let path = error.path.display();
    match error.kind {
        EntryErrorKind::Missing => format!("Unix mod compatibility path is missing: {path}"),
        EntryErrorKind::NotDirect => {
            format!("Unix mod compatibility path is not a direct file/directory: {path}")
        }
        EntryErrorKind::Io(io_err) => {
            format!("Inspect Unix mod compatibility path {path}: {io_err}")
        }
    }
}

#[cfg_attr(not(unix), allow(dead_code))]
#[derive(Clone, Copy)]
enum TokenKind {
    Word {
        start: usize,
        end: usize,
    },
    DelimitedString {
        content_start: usize,
        content_end: usize,
        token_end: usize,
    },
    PercentVar {
        token_end: usize,
    },
    LParen,
    RParen {
        token_end: usize,
    },
}

#[cfg_attr(not(unix), allow(dead_code))]
impl TokenKind {
    fn word_slice(self, input: &[u8]) -> Option<&[u8]> {
        match self {
            Self::Word { start, end } => Some(&input[start..end]),
            Self::DelimitedString { .. }
            | Self::PercentVar { .. }
            | Self::LParen
            | Self::RParen { .. } => None,
        }
    }

    fn literal_ident(self, input: &[u8]) -> Option<&[u8]> {
        let slice = match self {
            Self::Word { start, end } => &input[start..end],
            Self::DelimitedString {
                content_start,
                content_end,
                ..
            } => &input[content_start..content_end],
            Self::PercentVar { .. } | Self::LParen | Self::RParen { .. } => return None,
        };
        if slice.is_empty()
            || !slice
                .iter()
                .all(|&b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'#' | b'.' | b'-'))
        {
            return None;
        }
        Some(slice)
    }

    fn slice_eq_ci(self, input: &[u8], keyword: &[u8]) -> bool {
        self.word_slice(input)
            .is_some_and(|slice| slice.eq_ignore_ascii_case(keyword))
    }

    fn can_be_path_or_regex(self, input: &[u8]) -> bool {
        match self {
            Self::DelimitedString { .. } | Self::PercentVar { .. } => true,
            Self::Word { .. } => {
                !self.slice_eq_ci(input, b"^") && !self.slice_eq_ci(input, b"BEGIN")
            }
            Self::LParen | Self::RParen { .. } => false,
        }
    }

    fn trailing_slash_edit(self, input: &[u8]) -> Option<(usize, usize)> {
        let Self::DelimitedString {
            content_start,
            content_end,
            ..
        } = self
        else {
            return None;
        };
        let inner = &input[content_start..content_end];
        if inner.is_empty()
            || inner.contains(&b'\r')
            || inner.contains(&b'\n')
            || !inner.iter().any(|&b| b != b'/' && b != b'\\')
            || !matches!(inner.last(), Some(b'/' | b'\\'))
        {
            return None;
        }
        let mut trimmed_len = inner.len();
        while trimmed_len > 0 && matches!(inner[trimmed_len - 1], b'/' | b'\\') {
            trimmed_len -= 1;
        }
        if trimmed_len == inner.len() {
            None
        } else {
            Some((content_start + trimmed_len, content_end))
        }
    }
}

#[cfg_attr(not(unix), allow(dead_code))]
struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
}

#[cfg_attr(not(unix), allow(dead_code))]
impl<'a> Lexer<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    fn skip_ignored(&mut self) {
        while self.pos < self.input.len() {
            let b = self.input[self.pos];
            if b.is_ascii_whitespace() {
                self.pos += 1;
                continue;
            }
            if self.input[self.pos..].starts_with(b"//") {
                self.pos += 2;
                while self.pos < self.input.len() && self.input[self.pos] != b'\n' {
                    self.pos += 1;
                }
                continue;
            }
            if self.input[self.pos..].starts_with(b"/*") {
                self.pos += 2;
                let mut depth = 1usize;
                while self.pos < self.input.len() && depth > 0 {
                    if self.input[self.pos..].starts_with(b"/*") {
                        depth += 1;
                        self.pos += 2;
                    } else if self.input[self.pos..].starts_with(b"*/") {
                        depth -= 1;
                        self.pos += 2;
                    } else {
                        self.pos += 1;
                    }
                }
                continue;
            }
            if self.input[self.pos..].starts_with(b"<<<<<<<<") {
                self.pos += 8;
                while self.pos < self.input.len() {
                    if self.input[self.pos..].starts_with(b">>>>>>>>") {
                        self.pos += 8;
                        break;
                    }
                    self.pos += 1;
                }
                continue;
            }
            break;
        }
    }

    fn next_token(&mut self) -> Option<TokenKind> {
        self.skip_ignored();
        if self.pos >= self.input.len() {
            return None;
        }
        let rest = &self.input[self.pos..];
        if rest.starts_with(b"~~~~~") {
            let content_start = self.pos + 5;
            if let Some(rel_end) = self.input[content_start..]
                .windows(5)
                .position(|w| w == b"~~~~~")
            {
                let content_end = content_start + rel_end;
                self.pos = content_end + 5;
                return Some(TokenKind::DelimitedString {
                    content_start,
                    content_end,
                    token_end: self.pos,
                });
            }
            self.pos = self.input.len();
            return None;
        }
        let first = rest[0];
        if first == b'~' || first == b'"' {
            let content_start = self.pos + 1;
            if let Some(rel_end) = self.input[content_start..].iter().position(|&b| b == first) {
                let content_end = content_start + rel_end;
                self.pos = content_end + 1;
                return Some(TokenKind::DelimitedString {
                    content_start,
                    content_end,
                    token_end: self.pos,
                });
            }
            self.pos = self.input.len();
            return None;
        }
        if first == b'(' {
            self.pos += 1;
            return Some(TokenKind::LParen);
        }
        if first == b')' {
            self.pos += 1;
            return Some(TokenKind::RParen {
                token_end: self.pos,
            });
        }
        if first == b'%' {
            let mut scan = self.pos + 1;
            while scan < self.input.len() {
                let ch = self.input[scan];
                if ch == b'%' {
                    self.pos = scan + 1;
                    return Some(TokenKind::PercentVar {
                        token_end: self.pos,
                    });
                }
                if ch == b'\n' || ch == b'\r' {
                    break;
                }
                scan += 1;
            }
            // Fall back to a bareword token if not a single-line `%var%`.
        }
        let start = self.pos;
        while self.pos < self.input.len() {
            let b = self.input[self.pos];
            if b.is_ascii_whitespace()
                || b == b'~'
                || b == b'"'
                || b == b'%'
                || b == b'('
                || b == b')'
                || self.input[self.pos..].starts_with(b"//")
                || self.input[self.pos..].starts_with(b"/*")
                || self.input[self.pos..].starts_with(b"<<<<<<<<")
            {
                break;
            }
            self.pos += 1;
        }
        if self.pos == start {
            self.pos += 1;
        }
        Some(TokenKind::Word {
            start,
            end: self.pos,
        })
    }
}

/// Applies all Unix TP2/TPA/TPH/TPP compatibility normalizations in sequence without altering
/// comments, inlined files, non-UTF-8 bytes, or line endings. Returns `None` when nothing changed.
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) fn rewrite_unix_mod_script(input: &[u8]) -> Option<Vec<u8>> {
    match rewrite_bash_for_trailing_slashes(input) {
        Some(step1) => Some(rewrite_r2en_legacy_array_collisions(&step1).unwrap_or(step1)),
        None => rewrite_r2en_legacy_array_collisions(input),
    }
}

/// Removes trailing `/` and `\` characters from non-root directory operands in valid
/// `(ACTION|PATCH)_BASH_FOR ... BEGIN` statements without altering comments, inlined files,
/// non-UTF-8 bytes, or line endings. Returns `None` when nothing changed.
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) fn rewrite_bash_for_trailing_slashes(input: &[u8]) -> Option<Vec<u8>> {
    let mut lexer = Lexer::new(input);
    let mut edits: Vec<(usize, usize)> = Vec::new();

    while let Some(token) = lexer.next_token() {
        if !(token.slice_eq_ci(input, b"ACTION_BASH_FOR")
            || token.slice_eq_ci(input, b"PATCH_BASH_FOR"))
        {
            continue;
        }
        let mut pairs = 0usize;
        let mut pending: Vec<(usize, usize)> = Vec::new();
        let mut valid = false;

        while let Some(tok) = lexer.next_token() {
            if tok.slice_eq_ci(input, b"BEGIN") {
                valid = pairs > 0;
                break;
            }
            if tok.slice_eq_ci(input, b"EXACT_MATCH")
                || tok.slice_eq_ci(input, b"CASE_SENSITIVE")
                || tok.slice_eq_ci(input, b"CASE_INSENSITIVE")
            {
                if pairs == 0 {
                    break;
                }
                continue;
            }
            if !tok.can_be_path_or_regex(input) {
                break;
            }
            let dir_tok = tok;
            let mut regexp_tok = None;
            while let Some(next) = lexer.next_token() {
                if next.slice_eq_ci(input, b"EXACT_MATCH")
                    || next.slice_eq_ci(input, b"CASE_SENSITIVE")
                    || next.slice_eq_ci(input, b"CASE_INSENSITIVE")
                {
                    continue;
                }
                regexp_tok = Some(next);
                break;
            }
            let Some(regexp_tok) = regexp_tok else {
                break;
            };
            if !regexp_tok.can_be_path_or_regex(input) {
                break;
            }
            if let Some(edit) = dir_tok.trailing_slash_edit(input) {
                pending.push(edit);
            }
            pairs += 1;
        }
        if valid {
            edits.extend(pending);
        }
    }

    if edits.is_empty() {
        return None;
    }

    let removed_bytes: usize = edits.iter().map(|(start, end)| end - start).sum();
    let mut output = Vec::with_capacity(input.len() - removed_bytes);
    let mut cursor = 0usize;
    for (cut_start, cut_end) in edits {
        output.extend_from_slice(&input[cursor..cut_start]);
        cursor = cut_end;
    }
    output.extend_from_slice(&input[cursor..]);
    Some(output)
}

#[cfg_attr(not(unix), allow(dead_code))]
const MAX_PATCH_EXP_DEPTH: usize = 64;

#[cfg_attr(not(unix), allow(dead_code))]
fn is_unary_patch_op(slice: &[u8]) -> bool {
    slice == b"-"
        || slice == b"!"
        || slice.eq_ignore_ascii_case(b"BNOT")
        || slice.eq_ignore_ascii_case(b"ABS")
}

#[cfg_attr(not(unix), allow(dead_code))]
fn is_binary_patch_op(slice: &[u8]) -> bool {
    slice == b"+"
        || slice == b"-"
        || slice == b"*"
        || slice == b"/"
        || slice.eq_ignore_ascii_case(b"MODULO")
        || slice.eq_ignore_ascii_case(b"BAND")
        || slice.eq_ignore_ascii_case(b"BOR")
        || slice.eq_ignore_ascii_case(b"BXOR")
        || slice.eq_ignore_ascii_case(b"BLSL")
        || slice.eq_ignore_ascii_case(b"BLSR")
        || slice.eq_ignore_ascii_case(b"BASR")
}

#[cfg_attr(not(unix), allow(dead_code))]
fn parse_patch_primary(
    tokens: &[TokenKind],
    input: &[u8],
    idx: usize,
    depth: usize,
) -> Option<(usize, usize)> {
    if idx >= tokens.len() || depth >= MAX_PATCH_EXP_DEPTH {
        return None;
    }
    match tokens[idx] {
        TokenKind::LParen => {
            let mut paren_depth = 1usize;
            let mut scan = idx + 1;
            while scan < tokens.len() {
                match tokens[scan] {
                    TokenKind::LParen => paren_depth += 1,
                    TokenKind::RParen { token_end } => {
                        paren_depth -= 1;
                        if paren_depth == 0 {
                            return Some((scan + 1, token_end));
                        }
                    }
                    _ => {}
                }
                scan += 1;
            }
            None
        }
        TokenKind::RParen { .. } => None,
        TokenKind::Word { start, end } => {
            let slice = &input[start..end];
            if is_unary_patch_op(slice) {
                parse_patch_primary(tokens, input, idx + 1, depth + 1)
            } else {
                Some((idx + 1, end))
            }
        }
        TokenKind::DelimitedString { token_end, .. } | TokenKind::PercentVar { token_end } => {
            Some((idx + 1, token_end))
        }
    }
}

#[cfg_attr(not(unix), allow(dead_code))]
fn parse_patch_exp(tokens: &[TokenKind], input: &[u8], start_idx: usize) -> Option<(usize, usize)> {
    let (mut next_idx, mut end_offset) = parse_patch_primary(tokens, input, start_idx, 0)?;
    while next_idx < tokens.len() {
        let Some(op_slice) = tokens[next_idx].word_slice(input) else {
            break;
        };
        if !is_binary_patch_op(op_slice) {
            break;
        }
        let Some((rhs_next, rhs_end)) = parse_patch_primary(tokens, input, next_idx + 1, 0) else {
            break;
        };
        next_idx = rhs_next;
        end_offset = rhs_end;
    }
    Some((next_idx, end_offset))
}

/// Inserts `CLEAR_ARRAY ~<name>~` immediately after legacy `READ_2DA_ENTRIES_NOW <name> <cols>`
/// statements when the same file both reads rows via `READ_2DA_ENTRY_FORMER <name>` and iterates
/// an associative array named `<name>` via `(ACTION_)PHP_EACH <name> AS ...`, avoiding WeiDU 251's
/// `(row, col)` associative-array population collision without breaking WeiDU-251-native arrays.
/// Returns `None` when nothing changed.
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) fn rewrite_r2en_legacy_array_collisions(input: &[u8]) -> Option<Vec<u8>> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    while let Some(tok) = lexer.next_token() {
        tokens.push(tok);
    }

    let mut former_names: HashSet<&[u8]> = HashSet::new();
    let mut php_each_names: HashSet<&[u8]> = HashSet::new();
    for (i, tok) in tokens.iter().enumerate() {
        if tok.slice_eq_ci(input, b"READ_2DA_ENTRY_FORMER") {
            if let Some(name) = tokens.get(i + 1).and_then(|t| t.literal_ident(input)) {
                former_names.insert(name);
            }
        } else if tok.slice_eq_ci(input, b"PHP_EACH") || tok.slice_eq_ci(input, b"ACTION_PHP_EACH")
        {
            if let (Some(name), Some(as_tok)) = (
                tokens.get(i + 1).and_then(|t| t.literal_ident(input)),
                tokens.get(i + 2),
            ) {
                if as_tok.slice_eq_ci(input, b"AS") {
                    php_each_names.insert(name);
                }
            }
        }
    }

    if former_names.is_empty() || php_each_names.is_empty() {
        return None;
    }

    let mut inserts: Vec<(usize, &[u8])> = Vec::new();
    let mut i = 0usize;
    while i < tokens.len() {
        if !tokens[i].slice_eq_ci(input, b"READ_2DA_ENTRIES_NOW") {
            i += 1;
            continue;
        }
        let Some(name) = tokens.get(i + 1).and_then(|t| t.literal_ident(input)) else {
            i += 1;
            continue;
        };
        if !former_names.contains(name) || !php_each_names.contains(name) {
            i += 1;
            continue;
        }
        let Some((next_idx, insert_offset)) = parse_patch_exp(&tokens, input, i + 2) else {
            i += 1;
            continue;
        };
        let already_cleared = tokens
            .get(next_idx)
            .is_some_and(|tok| tok.slice_eq_ci(input, b"CLEAR_ARRAY"))
            && tokens
                .get(next_idx + 1)
                .and_then(|tok| tok.literal_ident(input))
                == Some(name);
        if !already_cleared {
            inserts.push((insert_offset, name));
        }
        i = next_idx;
    }

    if inserts.is_empty() {
        return None;
    }

    let added_bytes: usize = inserts.iter().map(|(_, name)| 15 + name.len()).sum();
    let mut output = Vec::with_capacity(input.len() + added_bytes);
    let mut cursor = 0usize;
    for (offset, name) in inserts {
        output.extend_from_slice(&input[cursor..offset]);
        output.extend_from_slice(b" CLEAR_ARRAY ~");
        output.extend_from_slice(name);
        output.push(b'~');
        cursor = offset;
    }
    output.extend_from_slice(&input[cursor..]);
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::{
        rewrite_bash_for_trailing_slashes, rewrite_r2en_legacy_array_collisions,
        rewrite_unix_mod_script,
    };

    #[test]
    fn rewrites_yeslick_style_action_and_patch_bash_for_trailing_slashes_and_backslashes() {
        let input = concat!(
            "ACTION_IF is_ee BEGIN\r\n",
            "\tACTION_BASH_FOR ~%MOD_FOLDER%/bam/~ ~^.+\\.bam$~ BEGIN\r\n",
            "\t\tCOPY ~%BASH_FOR_FILESPEC%~ ~override/%BASH_FOR_FILE%~\r\n",
            "\tEND\r\n",
            "\tPATCH_BASH_FOR \"sub\\dir\\\\\" ~.*~ BEGIN\r\n",
            "\tEND\r\n",
            "END\r\n",
        );
        let rewritten = rewrite_bash_for_trailing_slashes(input.as_bytes()).unwrap();
        let text = String::from_utf8(rewritten.clone()).unwrap();
        assert_eq!(
            text,
            concat!(
                "ACTION_IF is_ee BEGIN\r\n",
                "\tACTION_BASH_FOR ~%MOD_FOLDER%/bam~ ~^.+\\.bam$~ BEGIN\r\n",
                "\t\tCOPY ~%BASH_FOR_FILESPEC%~ ~override/%BASH_FOR_FILE%~\r\n",
                "\tEND\r\n",
                "\tPATCH_BASH_FOR \"sub\\dir\" ~.*~ BEGIN\r\n",
                "\tEND\r\n",
                "END\r\n",
            )
        );
        assert!(rewrite_bash_for_trailing_slashes(&rewritten).is_none());
    }

    #[test]
    fn supports_multi_pair_modifiers_and_five_tilde_strings_while_preserving_non_utf8_bytes() {
        let mut input = Vec::new();
        input.extend_from_slice(b"// caf\xe9 comment\r\n");
        input.extend_from_slice(
            b"ACTION_BASH_FOR ~dir/one/~ EXACT_MATCH ~\\.txt$~ ~~~~~dir/two///~~~~~ CASE_INSENSITIVE \"\\.2da$\" BEGIN\r\n",
        );
        input.extend_from_slice(b"\tPRINT ~caf\xe9~\r\nEND\r\n");

        let rewritten = rewrite_bash_for_trailing_slashes(&input).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(b"// caf\xe9 comment\r\n");
        expected.extend_from_slice(
            b"ACTION_BASH_FOR ~dir/one~ EXACT_MATCH ~\\.txt$~ ~~~~~dir/two~~~~~ CASE_INSENSITIVE \"\\.2da$\" BEGIN\r\n",
        );
        expected.extend_from_slice(b"\tPRINT ~caf\xe9~\r\nEND\r\n");
        assert_eq!(rewritten, expected);
    }

    #[test]
    fn leaves_comments_strings_inlined_files_root_slashes_and_incomplete_statements_untouched() {
        let input = concat!(
            "// ACTION_BASH_FOR ~dir/comment/~ ~.*~ BEGIN\n",
            "/* outer /* ACTION_BASH_FOR ~dir/nested/~ ~.*~ BEGIN */ still comment ACTION_BASH_FOR ~dir/still/~ ~.*~ BEGIN */\n",
            "OUTER_SPRINT s \"ACTION_BASH_FOR ~%resource_loc%/\\1~ ~^.+\\.wav$~ BEGIN COPY ~foo~ ~bar~ END\"\n",
            "<<<<<<<< .../inline.tpa\n",
            "ACTION_BASH_FOR ~dir/inlined/~ ~.*~ BEGIN\n",
            ">>>>>>>>\n",
            "ACTION_BASH_FOR ~/~ ~.*~ BEGIN END\n",
            "ACTION_BASH_FOR ~\\\\~ ~.*~ BEGIN END\n",
            "ACTION_BASH_FOR ~dir/no_begin/~ ~.*~\n",
        );
        assert!(rewrite_bash_for_trailing_slashes(input.as_bytes()).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn unix_prepare_rewrites_mixed_case_mod_tree_idempotently_and_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        std::fs::create_dir_all(root.join("YeslickNPC/lib")).unwrap();
        std::fs::create_dir_all(root.join("YeslickNPC/backup")).unwrap();
        std::fs::write(
            root.join("YeslickNPC/lib/main_component.tpa"),
            b"ACTION_BASH_FOR ~%MOD_FOLDER%/bam/~ ~^.+\\.bam$~ BEGIN\nEND\n",
        )
        .unwrap();
        std::fs::write(
            root.join("YeslickNPC/backup/untouched.tpa"),
            b"ACTION_BASH_FOR ~backup/sub/~ ~.*~ BEGIN\nEND\n",
        )
        .unwrap();

        let first = super::prepare(super::UnixModCompatibilityInput {
            mod_id: "yeslicknpc-bg2",
            tp2: "yeslicknpc/yeslicknpc.tp2",
            target_root: root,
        })
        .unwrap()
        .expect("first run should patch mixed-case main_component.tpa");
        assert_eq!(first.fix_id, "weidu-unix-mod-compatibility-v1");
        assert_eq!(first.patched_files.len(), 1);
        assert_eq!(
            first.patched_files[0].relative_path,
            "YeslickNPC/lib/main_component.tpa"
        );
        assert_eq!(
            std::fs::read_to_string(root.join("YeslickNPC/lib/main_component.tpa")).unwrap(),
            "ACTION_BASH_FOR ~%MOD_FOLDER%/bam~ ~^.+\\.bam$~ BEGIN\nEND\n"
        );
        assert!(
            std::fs::read_to_string(root.join("YeslickNPC/backup/untouched.tpa"))
                .unwrap()
                .contains("~backup/sub/~")
        );

        let second = super::prepare(super::UnixModCompatibilityInput {
            mod_id: "yeslicknpc-bg2",
            tp2: "yeslicknpc/yeslicknpc.tp2",
            target_root: root,
        })
        .unwrap();
        assert!(second.is_none());

        symlink(
            root.join("YeslickNPC/backup/untouched.tpa"),
            root.join("YeslickNPC/lib/evil.tpa"),
        )
        .unwrap();
        let err = super::prepare(super::UnixModCompatibilityInput {
            mod_id: "yeslicknpc-bg2",
            tp2: "yeslicknpc/yeslicknpc.tp2",
            target_root: root,
        })
        .unwrap_err();
        assert!(err.contains("not a direct file/directory"), "{err}");
    }

    #[test]
    fn rewrites_ds_ui_extra_spells_and_bethany_r2en_legacy_array_collisions() {
        let input = concat!(
            "// Detectable Spells + ui_extra_spells + Bethany\r\n",
            "COPY_EXISTING - stats.ids override\r\n",
            "  READ_2DA_ENTRIES_NOW stats 2\r\n",
            "  FOR (i=0;i<stats;i+=1) BEGIN\r\n",
            "    READ_2DA_ENTRY_FORMER stats i 0 num\r\n",
            "    SPRINT $stats(\"%num%\") ~val~\r\n",
            "  END\r\n",
            "ACTION_PHP_EACH stats AS ind => res BEGIN\r\n",
            "END\r\n",
            "COPY - ~dw_ext_spell.ids~ nowhere\r\n",
            "  CLEAR_ARRAY ext_spell_data\r\n",
            "  READ_2DA_ENTRIES_NOW ext_spell_data 2\r\n",
            "  READ_2DA_ENTRY_FORMER ext_spell_data row 1 id_here\r\n",
            "ACTION_PHP_EACH ext_spell_data AS id_here => discard BEGIN END\r\n",
            "COPY_EXISTING tooltip.2da override\r\n",
            "  READ_2DA_ENTRIES_NOW rn cn\r\n",
            "  READ_2DA_ENTRY_FORMER rn k 0 itm\r\n",
            "  PHP_EACH rn AS i => r BEGIN END\r\n",
        );
        let rewritten = rewrite_r2en_legacy_array_collisions(input.as_bytes()).unwrap();
        let text = String::from_utf8(rewritten.clone()).unwrap();
        assert_eq!(
            text,
            concat!(
                "// Detectable Spells + ui_extra_spells + Bethany\r\n",
                "COPY_EXISTING - stats.ids override\r\n",
                "  READ_2DA_ENTRIES_NOW stats 2 CLEAR_ARRAY ~stats~\r\n",
                "  FOR (i=0;i<stats;i+=1) BEGIN\r\n",
                "    READ_2DA_ENTRY_FORMER stats i 0 num\r\n",
                "    SPRINT $stats(\"%num%\") ~val~\r\n",
                "  END\r\n",
                "ACTION_PHP_EACH stats AS ind => res BEGIN\r\n",
                "END\r\n",
                "COPY - ~dw_ext_spell.ids~ nowhere\r\n",
                "  CLEAR_ARRAY ext_spell_data\r\n",
                "  READ_2DA_ENTRIES_NOW ext_spell_data 2 CLEAR_ARRAY ~ext_spell_data~\r\n",
                "  READ_2DA_ENTRY_FORMER ext_spell_data row 1 id_here\r\n",
                "ACTION_PHP_EACH ext_spell_data AS id_here => discard BEGIN END\r\n",
                "COPY_EXISTING tooltip.2da override\r\n",
                "  READ_2DA_ENTRIES_NOW rn cn CLEAR_ARRAY ~rn~\r\n",
                "  READ_2DA_ENTRY_FORMER rn k 0 itm\r\n",
                "  PHP_EACH rn AS i => r BEGIN END\r\n",
            )
        );
        assert!(rewrite_r2en_legacy_array_collisions(&rewritten).is_none());
    }

    #[test]
    fn parses_complex_r2en_cols_expressions_without_eating_trailing_statements() {
        let mut input = Vec::new();
        input.extend_from_slice(b"// caf\xe9\n");
        input.extend_from_slice(
            concat!(
                "READ_2DA_ENTRIES_NOW ~file~ num_col - 1\n",
                "READ_2DA_ENTRIES_NOW parenthesized (cols + 1)\n",
                "READ_2DA_ENTRIES_NOW \"quoted\" \"%cols%\"\n",
                "READ_2DA_ENTRIES_NOW data cols SET found = 0\n",
                "READ_2DA_ENTRY_FORMER file 0 0 a\n",
                "READ_2DA_ENTRY_FORMER parenthesized 0 0 b\n",
                "READ_2DA_ENTRY_FORMER quoted 0 0 c\n",
                "READ_2DA_ENTRY_FORMER data 0 0 d\n",
                "PHP_EACH file AS k => v BEGIN END\n",
                "PHP_EACH parenthesized AS k => v BEGIN END\n",
                "PHP_EACH quoted AS k => v BEGIN END\n",
                "PHP_EACH data AS k => v BEGIN END\n",
            )
            .as_bytes(),
        );
        let rewritten = rewrite_r2en_legacy_array_collisions(&input).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(b"// caf\xe9\n");
        expected.extend_from_slice(
            concat!(
                "READ_2DA_ENTRIES_NOW ~file~ num_col - 1 CLEAR_ARRAY ~file~\n",
                "READ_2DA_ENTRIES_NOW parenthesized (cols + 1) CLEAR_ARRAY ~parenthesized~\n",
                "READ_2DA_ENTRIES_NOW \"quoted\" \"%cols%\" CLEAR_ARRAY ~quoted~\n",
                "READ_2DA_ENTRIES_NOW data cols CLEAR_ARRAY ~data~ SET found = 0\n",
                "READ_2DA_ENTRY_FORMER file 0 0 a\n",
                "READ_2DA_ENTRY_FORMER parenthesized 0 0 b\n",
                "READ_2DA_ENTRY_FORMER quoted 0 0 c\n",
                "READ_2DA_ENTRY_FORMER data 0 0 d\n",
                "PHP_EACH file AS k => v BEGIN END\n",
                "PHP_EACH parenthesized AS k => v BEGIN END\n",
                "PHP_EACH quoted AS k => v BEGIN END\n",
                "PHP_EACH data AS k => v BEGIN END\n",
            )
            .as_bytes(),
        );
        assert_eq!(rewritten, expected);
    }

    #[test]
    fn leaves_native_r2en_arrays_and_comments_untouched_and_combines_unix_rewrites() {
        let no_former = concat!(
            "READ_2DA_ENTRIES_NOW modern 2\n",
            "PHP_EACH modern AS k => v BEGIN END\n",
        );
        assert!(rewrite_r2en_legacy_array_collisions(no_former.as_bytes()).is_none());

        let no_php_each = concat!(
            "READ_2DA_ENTRIES_NOW plain 2\n",
            "READ_2DA_ENTRY_FORMER plain 0 0 x\n",
        );
        assert!(rewrite_r2en_legacy_array_collisions(no_php_each.as_bytes()).is_none());

        let commented_and_inlined = concat!(
            "// READ_2DA_ENTRIES_NOW stats 2\n",
            "/* READ_2DA_ENTRY_FORMER stats 0 0 x */\n",
            "<<<<<<<< .../inline.tpa\n",
            "READ_2DA_ENTRIES_NOW stats 2\n",
            ">>>>>>>>\n",
            "ACTION_PHP_EACH stats AS k => v BEGIN END\n",
        );
        assert!(rewrite_r2en_legacy_array_collisions(commented_and_inlined.as_bytes()).is_none());

        let combined = concat!(
            "ACTION_BASH_FOR ~sub/~ ~.*~ BEGIN END\n",
            "READ_2DA_ENTRIES_NOW stats 2\n",
            "READ_2DA_ENTRY_FORMER stats 0 0 num\n",
            "ACTION_PHP_EACH stats AS k => v BEGIN END\n",
        );
        let rewritten = rewrite_unix_mod_script(combined.as_bytes()).unwrap();
        assert_eq!(
            String::from_utf8(rewritten.clone()).unwrap(),
            concat!(
                "ACTION_BASH_FOR ~sub~ ~.*~ BEGIN END\n",
                "READ_2DA_ENTRIES_NOW stats 2 CLEAR_ARRAY ~stats~\n",
                "READ_2DA_ENTRY_FORMER stats 0 0 num\n",
                "ACTION_PHP_EACH stats AS k => v BEGIN END\n",
            )
        );
        assert!(rewrite_unix_mod_script(&rewritten).is_none());
    }
}
