use crate::repo::Repo;

use super::findable::Findable;
use super::serializable::Serializable;

use super::mode::Mode;

/// A `tree` in git describes the state of a work tree.
///
/// At a high-level, a tree is analogous to a directory and a blob is analogous
/// to a file. A tree object simply stores a mapping of objects to paths, and
/// those objects are just hashes -- they could be a blob (file) or another
/// tree (directory).
///
/// ## Example
/// | Mode     | Hash                                       | Path           |
/// | -------- | ------------------------------------------ | -------------- |
/// | `100644` | `ada8be928d8354117a641ab97802e5bb42740ae9` | `.gitignore`   |
/// | `040000` | `f3ba09e336a7219a14b2d92ef71e41414d50d1d9` | `.vscode`      |
/// | `100644` | `510f3a63b99448773a54ef641ef4c209332b0ccd` | `Cargo.toml`   |
/// | `100644` | `aa37048a26b432c8d10febeec3d0478eb2f38b69` | `README.md`    |
/// | `100644` | `6f2e075152c9a14b36d532560e9d1893d6052f4e` | `rustfmt.toml` |     
/// | `040000` | `4b1208faa3d3fe86fae5f7e6a8363c3f765b3ddd` | `src`          |
/// | `040000` | `cdabf2fe75f115689b6bd94832eb340e4dfd6891` | `tests`        |
pub struct Tree {
  bytes: Vec<u8>,
  entries: Vec<TreeEntry>,
  format: String,
  repo: Repo,
}

impl Tree {
  pub fn new(repo: Repo, data: &[u8]) -> Self {
    let mut new_tree: Self = Self {
      bytes: Vec::default(),
      entries: Vec::default(),
      format: String::from("tree"),
      repo,
    };
    new_tree.deserialize(data);
    new_tree
  }

  pub fn entries(&self) -> &Vec<TreeEntry> {
    &self.entries
  }
}

impl Serializable for Tree {
  fn serialize(&self) -> &[u8] {
    &self.bytes
  }

  fn deserialize(&mut self, data: &[u8]) {
    self.bytes = data.to_vec();
    let mut offset: usize = 0;
    while offset < self.bytes.len() {
      let entry: TreeEntry = TreeEntry::from_bytes(&self.bytes, offset);
      offset += entry.len;
      self.entries.push(entry);
    }
  }

  fn format(&self) -> &String {
    &self.format
  }

  fn repo(&self) -> &Repo {
    &self.repo
  }
}

/// A single tree entry.
#[derive(Debug)]
pub struct TreeEntry {
  pub mode: Mode,
  pub path: String,
  pub hash: String,
  pub len: usize,
}

impl TreeEntry {
  /// Constructs a new TreeEntry from raw bytes starting at offset.
  ///
  /// An entry in the bytes is formatted as: `[mode] 0x20 [path] 0x00 [sha-1]`
  pub fn from_bytes(raw: &[u8], offset: usize) -> Self {
    // Search for the first space after offset (a space is 0x20).
    let maybe_space = raw.find(b' ', offset);
    let space = match maybe_space {
      // mode should be either a 5 or 6 digit number
      Some(i) if i == offset + 5 || i == offset + 6 => i,
      _ => panic!("Failed to create TreeEntry: inconsistent input"),
    };

    // Extract the mode as a string, convert to Mode enum
    let mode = String::from_utf8(raw[offset..space].to_vec()).unwrap();
    let mode = mode.parse::<usize>().unwrap();
    let mode: Mode = Mode::try_from(mode).expect("unable to parse file mode");

    // Find the null-terminator of the path
    let maybe_null = raw.find(b'\0', space);
    let null = match maybe_null {
      Some(i) => i,
      _ => panic!("Failed to create TreeEntry: inconsistent input"),
    };
    let path = String::from_utf8(raw[space + 1..null].to_vec()).unwrap();

    // Read out the hash and convert it to a hex string (20 bytes)
    let hash = hex::encode(&raw[null + 1..null + 21]);
    let len = null + 21 - offset;
    Self {
      mode,
      path,
      hash,
      len,
    }
  }
}
