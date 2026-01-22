use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{BufRead, Cursor, Result as IoResult, Seek, Write};
use std::io::{BufReader, BufWriter, Error as IoError, ErrorKind};
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub trait ReadSeek: Seek + BufRead {}
impl<R: BufRead + Seek> ReadSeek for R {}

#[derive(Clone)]
pub struct RealFileSystem {
    pub root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct FakeFileSystem {
    root: PathBuf,
    mapping: Rc<RefCell<HashMap<PathBuf, Vec<u8>>>>,
}

pub trait FileSystem {
    fn duplicate(&self) -> Box<dyn FileSystem>;
    fn subsystem(&self, path: &Path) -> Box<dyn FileSystem>;
    fn exists(&self, path: &Path) -> bool;
    fn read(&self, path: &Path, f: &mut dyn FnMut(&mut dyn ReadSeek) -> IoResult<()>) -> IoResult<()>;
    fn write(&self, path: &Path, f: &mut dyn FnMut(&mut dyn Write) -> IoResult<()>) -> IoResult<()>;
    fn full_path_for(&self, path: &Path) -> PathBuf;
    fn files(&self) -> Vec<PathBuf>;
    fn remove(&self, path: &Path) -> IoResult<()>;
    fn is_empty(&self) -> bool {
        self.files().is_empty()
    }
    fn copy(&self, from: &Path, to: &Path) -> IoResult<()> {
        self.write(to, &mut |writer| {
            self.read(from, &mut |reader| {
                ::std::io::copy(reader, writer).map(|_| ())
            })
        })
    }
}

impl FakeFileSystem {
    pub fn new() -> Self {
        FakeFileSystem {
            root: PathBuf::from("/"),
            mapping: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl FileSystem for RealFileSystem {
    fn subsystem(&self, path: &Path) -> Box<dyn FileSystem> {
        assert!(path.is_relative(), "path must be relative");
        let mut new = self.clone();
        new.root.push(path);
        Box::new(new)
    }

    fn remove(&self, path: &Path) -> IoResult<()> {
        let path = self.root.join(path);
        ::std::fs::remove_file(path)
    }

    fn duplicate(&self) -> Box<dyn FileSystem> {
        Box::new(self.clone())
    }

    fn exists(&self, path: &Path) -> bool {
        let path = self.root.join(path);
        path.exists()
    }

    fn read(&self, path: &Path, f: &mut dyn FnMut(&mut dyn ReadSeek) -> IoResult<()>) -> IoResult<()> {
        let path = self.root.join(path);
        match File::open(path) {
            Ok(file) => {
                let mut file = BufReader::new(file);
                f(&mut file)
            }
            Err(e) => Err(e),
        }
    }

    fn write(&self, path: &Path, f: &mut dyn FnMut(&mut dyn Write) -> IoResult<()>) -> IoResult<()> {
        let path = self.root.join(path);
        create_dir_all(path.parent().unwrap())?;

        match File::create(path) {
            Ok(file) => {
                let mut file = BufWriter::new(file);
                f(&mut file)
            }
            Err(e) => Err(e),
        }
    }

    fn full_path_for(&self, path: &Path) -> PathBuf {
        self.root.join(path)
    }

    fn files(&self) -> Vec<PathBuf> {
        ::walkdir::WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|p| p.path().to_owned())
            .filter_map(|p| p.strip_prefix(&self.root).ok().map(|p| p.to_owned()))
            .collect()
    }
}

impl FileSystem for FakeFileSystem {
    fn subsystem(&self, path: &Path) -> Box<dyn FileSystem> {
        assert!(path.is_relative(), "path must be relative");
        let mut new = self.clone();
        new.root.push(path);
        Box::new(new)
    }
    fn duplicate(&self) -> Box<dyn FileSystem> {
        Box::new(self.clone())
    }

    fn exists(&self, path: &Path) -> bool {
        let path = self.root.join(path);
        self.mapping.borrow().contains_key(&path)
    }

    fn remove(&self, path: &Path) -> IoResult<()> {
        let path = self.root.join(path);
        self.mapping.borrow_mut().remove(&path);
        Ok(())
    }

    fn read(&self, path: &Path, f: &mut dyn FnMut(&mut dyn ReadSeek) -> IoResult<()>) -> IoResult<()> {
        let path = self.root.join(path);

        let contents = match self.mapping.borrow().get(&path) {
            Some(contents) => contents.clone(),
            None => {
                return Err(IoError::new(
                    ErrorKind::NotFound,
                    format!("{:?} does not exist", path),
                ))
            }
        };

        f(&mut Cursor::new(&contents[..]))
    }

    fn write(&self, path: &Path, f: &mut dyn FnMut(&mut dyn Write) -> IoResult<()>) -> IoResult<()> {
        let path = self.root.join(path);

        let mut contents = vec![];
        f(&mut contents)?;

        self.mapping.borrow_mut().insert(path, contents);
        Ok(())
    }

    fn full_path_for(&self, path: &Path) -> PathBuf {
        self.root.join(path)
    }

    fn files(&self) -> Vec<PathBuf> {
        let root = self.root.clone();
        self.mapping
            .borrow()
            .keys()
            .filter_map(|p| p.strip_prefix(&root).ok())
            .map(|p| p.into())
            .collect()
    }
}
