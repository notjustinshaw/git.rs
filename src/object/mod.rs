pub(crate) mod blob;
pub(crate) mod commit;
pub(crate) mod findable;
pub(crate) mod mail_map;
pub(crate) mod mode;
pub(crate) mod refs;
pub(crate) mod serializable;
pub(crate) mod tag;
pub(crate) mod tree;

use crate::crypto;
use crate::object::blob::Blob;
use crate::object::commit::Commit;
use crate::object::findable::Findable;
use crate::object::serializable::Serializable;
use crate::object::tree::Tree;
use crate::repo::{repo_file, Repo};
use std::fs::{self, File};
use std::io::prelude::*;

use self::tag::Tag;

/// A git object.
///
/// In git, objects are a generic structure used for a lot of various things. At
/// its core, an object is a compressed file in the `.git` directory whose path
/// is determined by its contents. The path is computed using the SHA-1 hash of
/// the payload split into the first 2 bytes which represent the object's
/// directory followed by the last 30 bytes which represent the filename.
///
/// The path to the object `e673d1b7eaa0aa01b5bc2442d570a765bdaae751` would be
/// `.git/objects/e6/73d1b7eaa0aa01b5bc2442d570a765bdaae751`. Inside that file
/// would be a zlib compressed payload prepended with an object header.
///
/// The first 48 bytes of a commit object might look like this:
///
/// ```text
/// 00000000  63 6f 6d 6d 69 74 20 31  30 38 36 00 74 72 65 65  |commit 1086.tree|
/// 00000010  20 32 39 66 66 31 36 63  39 63 31 34 65 32 36 35  | 29ff16c9c14e265|
/// 00000020  32 62 32 32 66 38 62 37  38 62 62 30 38 61 35 61  |2b22f8b78bb08a5a|
/// ```
///
/// An object starts with a header that specifies its type: `blob`, `commit`,
/// `tag` or `tree`. This header is followed by an ASCII space (0x20), then the
/// size of the object in bytes as an ASCII number, then null (0x00) (the null
/// byte), then the contents of the object.

/// Reads object object_id from the repository repo and returns an object
/// whose exact type depends on the object read from memory.
pub fn read(
  repo: Repo,
  hash: &str,
  typename: Option<&str>,
) -> Result<Box<dyn Serializable>, String> {
  let directories = ["objects", &hash[0..2], &hash[2..]];
  let path = match repo_file(&repo.git_dir, &directories, false) {
    Some(p) => p,
    None => return Err(format!("object not found {}", hash)),
  };
  if let Ok(file) = fs::read(path) {
    let raw = crypto::decompress(&file)?;

    // Read the object type
    let first_space: usize = raw.find(b' ', 0).unwrap();
    let object_type: &str = &String::from_utf8(raw[0..first_space].to_vec()).unwrap();
    match typename {
      Some(name) if object_type != name => {
        return Err(format!("invalid object type \"{}\"", typename.unwrap()))
      }
      _ => (),
    }

    // Read and validate the object size
    let null_byte: usize = raw.find(b'\0', 0).unwrap();
    let object_size: usize = String::from_utf8(raw[first_space + 1..null_byte].to_vec())
      .unwrap()
      .parse::<usize>()
      .unwrap();

    if object_size != raw.len() - null_byte - 1 {
      return Err("size does not match size of raw data".to_string());
    }

    let payload = &raw[null_byte + 1..];
    match object_type {
      "blob" => Ok(Box::new(Blob::new(repo, payload))),
      "commit" => Ok(Box::new(Commit::new(repo, payload))),
      "tag" => Ok(Box::new(Tag::new(repo, payload))),
      "tree" => Ok(Box::new(Tree::new(repo, payload))),
      _ => Err(format!("unsupported type \"{}\"", object_type)),
    }
  } else {
    Err("object not found".to_string())
  }
}

/// Writes an object to the repository.
///
/// The object is written to the repository that the object represents. If the
/// dry_run flag is set to true, the hash will be calculated but not written
/// to the directory.
pub fn write(object: &dyn Serializable, dry_run: bool) -> Result<String, String> {
  let payload = object.serialize();
  let header = format!("{} {}\0", object.format(), payload.len());
  let data = [header.as_bytes(), payload].concat();
  let hash = crypto::sha_1(&data);

  if !dry_run {
    let directories = ["objects", &hash[0..2], &hash[2..]];
    let path = repo_file(&object.repo().git_dir, &directories, true);
    let mut file = File::create(path.unwrap()).unwrap();
    let compressed_data = crypto::compress(&data)?;
    file.write_all(&compressed_data[..]).unwrap();
  }
  Ok(hash)
}

/// Finds object.
pub fn find_object<'a>(_repo: Repo, name: &'a str, _type: Option<&str>, _follow: bool) -> &'a str {
  name
}
