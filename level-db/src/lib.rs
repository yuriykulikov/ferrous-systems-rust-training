use std::ffi::CString;
use std::path::Path;
use std::ptr::{null_mut};

use anyhow::Error;
use leveldb_sys::{leveldb_close, leveldb_free};
use leveldb_sys::leveldb_get;
use leveldb_sys::leveldb_open;
use leveldb_sys::leveldb_options_create;
use leveldb_sys::leveldb_options_destroy;
use leveldb_sys::leveldb_options_set_create_if_missing;
use leveldb_sys::leveldb_options_t;
use leveldb_sys::leveldb_put;
use leveldb_sys::leveldb_readoptions_create;
use leveldb_sys::leveldb_readoptions_destroy;
use leveldb_sys::leveldb_t;
use leveldb_sys::leveldb_writeoptions_create;
use leveldb_sys::leveldb_writeoptions_destroy;

/// User-friendly wrapper for LevelDB
///
/// ## LevelDb
/// fast key-value storage library that provides an ordered mapping from string keys to string values.
///
#[derive(Debug)]
pub struct LevelDb {
    delegate: *mut leveldb_t,
    path: String,
}

impl LevelDb {
    /// Opens the database using the given [path]. Dropping this reference closes the database.
    pub fn open(path: &Path) -> Result<LevelDb, Error> {
        unsafe {
            let path_as_string = path.to_str()
                .ok_or_else(|| Error::msg("Invalid path"))?;

            let name = CString::new(path_as_string)?.as_ptr();

            let options: *mut leveldb_options_t = leveldb_options_create();
            leveldb_options_set_create_if_missing(options, 1);
            let mut errptr = null_mut();

            // let name = CString::new(path.to_str()).unwrap().into_raw();
            let level_db_instance: *mut leveldb_t = leveldb_open(options, name, &mut errptr);
            leveldb_options_destroy(options);

            if !errptr.is_null() {
                let err_str = CString::from_raw(errptr).to_str().unwrap().to_owned();
                println!("Error: {}", err_str);
                return Err(Error::msg(err_str));
            }

            if level_db_instance.is_null() {
                return Err(Error::msg("leveldb_open returned null!"));
            }

            Ok(
                LevelDb {
                    delegate: level_db_instance,
                    path: path.to_str().unwrap().to_owned(),
                }
            )
        }
    }

    /// Put the [key]-[value] pair into the storage
    pub fn put(&self, key: &str, value: &str) -> Result<(), Error> {
        unsafe {
            let mut errptr = null_mut();
            // pub fn leveldb_put(db: *mut leveldb_t, options: *const leveldb_writeoptions_t, key: *const c_char, keylen: size_t, val: *const c_char, vallen: size_t, errptr: *mut *mut c_char);
            let write_options = leveldb_writeoptions_create();
            leveldb_put(
                self.delegate,
                write_options,
                key.as_ptr() as *const i8,
                key.len(),
                value.as_ptr() as *const i8,
                value.len(),
                &mut errptr,
            );
            leveldb_writeoptions_destroy(write_options);

            if errptr.is_null() {
                Ok(())
            } else {
                Err(Error::msg(CString::from_raw(errptr).to_str().unwrap().to_owned()))
            }
        }
    }

    /// Get the value for the corresponding [key] from the storage
    pub fn get(&self, key: &str) -> Result<Option<String>, Error> {
        unsafe {
            let read_options = leveldb_readoptions_create();
            let mut errptr = null_mut();
            let mut len: usize = 0;
            let get = leveldb_get(
                self.delegate,
                read_options,
                key.as_bytes().as_ptr() as *const i8,
                key.len(),
                &mut len,
                &mut errptr,
            );
            leveldb_readoptions_destroy(read_options);
            if get.is_null() {
                Ok(None)
            } else if errptr.is_null() {
                let slice = std::slice::from_raw_parts(get as *mut u8, len);
                let ret = std::str::from_utf8(slice).unwrap().to_owned();
                // let ret = Ok(Some(String::from_raw_parts(get as *mut u8, len, len)));
                // this line causes free(): double free detected in tcache 2
                leveldb_free(get as *mut libc::c_void);
                Ok(Some(ret))
            } else {
                Err(Error::msg(CString::from_raw(errptr).to_str().unwrap().to_owned()))
            }
        }
    }
}

impl Drop for LevelDb {
    fn drop(&mut self) {
        unsafe {
            leveldb_close(self.delegate);
            println!("LevelDb: dropped {}", self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::Error;
    use tempdir::TempDir;

    use crate::LevelDb;

    #[test]
    fn open_ok() -> Result<(), Error> {
        let path = TempDir::new("level_db").unwrap().path().join("open_ok");
        let res = LevelDb::open(&path);
        assert_eq!(res.is_ok(), true);
        Ok(())
    }

    #[test]
    fn when_open_fails_result_provides_a_useful_error_message() -> Result<(), Error> {
        let res = LevelDb::open(Path::new("\\"));
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap().to_string(), "IO error: /LOCK: Permission denied");
        Ok(())
    }

    #[test]
    fn values_stored_with_put_can_be_read_with_get() -> Result<(), Error> {
        let path = TempDir::new("level_db").unwrap().path().join("values_stored_with_put_can_be_read_with_get");
        let level_db = LevelDb::open(&path)?;
        level_db.put("key", "value")?;
        assert_eq!(level_db.get("key")?, Some("value".to_owned()));
        Ok(())
    }

    #[test]
    fn missing_key_returns_none() -> Result<(), Error> {
        let path = TempDir::new("level_db").unwrap().path().join("missing_key_returns_none");
        let level_db = LevelDb::open(&path)?;
        assert_eq!(level_db.get("key")?, None);
        Ok(())
    }

    #[test]
    fn empty_value_returns_some_with_empty_string() -> Result<(), Error> {
        let path = TempDir::new("level_db").unwrap().path().join("empty_value_returns_some_with_empty_string");
        let level_db = LevelDb::open(&path)?;
        level_db.put("key", "")?;
        assert_eq!(level_db.get("key")?, Some("".to_owned()));
        Ok(())
    }

    #[test]
    fn values_stored_with_put_can_be_read_with_get_2() -> Result<(), Error> {
        let path = TempDir::new("level_db").unwrap().path().join("values_stored_with_put_can_be_read_with_get_2");
        let level_db = LevelDb::open(&path)?;
        level_db.put("key", "value")?;
        level_db.put("key1", "value1")?;
        level_db.put("key2", "value2")?;
        level_db.put("key3", "value3")?;
        assert_eq!(level_db.get("key")?, Some("value".to_owned()));
        assert_eq!(level_db.get("key1")?, Some("value1".to_owned()));
        assert_eq!(level_db.get("key2")?, Some("value2".to_owned()));
        assert_eq!(level_db.get("key3")?, Some("value3".to_owned()));
        Ok(())
    }
}