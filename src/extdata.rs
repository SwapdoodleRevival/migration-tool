use std::{mem, os::raw::c_void};

use ctru_sys::{
    self, FS_Archive, FS_DirectoryEntry, FS_MediaType, FS_Path, FSDIR_Close, FSDIR_Read,
    FSFILE_Close, FSFILE_Read, FSFILE_Write, FSUSER_CloseArchive, FSUSER_CreateFile,
    FSUSER_DeleteFile, FSUSER_OpenArchive, FSUSER_OpenDirectory, FSUSER_OpenFile, Handle,
    MEDIATYPE_SD, PATH_BINARY, PATH_UTF16, R_FAILED, R_SUCCEEDED, fsMakePath,
};
use libdoodle::bpk1::BPK1File;

macro_rules! handle_error {
    ($res: expr) => {
        let res = $res;
        if R_FAILED(res) {
            panic!("Error {res}");
        }
    };
}

pub enum SwapdoodleRegion {
    EU,
    US,
    JP,
}

pub struct ExtdataArchive {
    pub archive: FS_Archive,
}

impl ExtdataArchive {
    pub fn open(region: SwapdoodleRegion) -> Result<ExtdataArchive, ()> {
        let title_id: u64 = match region {
            SwapdoodleRegion::EU => 0x00040000001A2E00,
            SwapdoodleRegion::US => 0x00040000001A2D00,
            SwapdoodleRegion::JP => 0x00040000001A2C00,
        };
        let extdata = (title_id as u32) >> 8;
        let path: [u32; 3] = [MEDIATYPE_SD.into(), extdata, 0];

        unsafe {
            let mut extdata_handle: FS_Archive = mem::zeroed();

            match R_SUCCEEDED(FSUSER_OpenArchive(
                &mut extdata_handle as *mut _,
                0x00000006, // ARCHIVE_EXTDATA
                FS_Path {
                    type_: PATH_BINARY,
                    size: 12,
                    data: &path as *const _ as *const c_void,
                },
            )) {
                true => Ok(ExtdataArchive {
                    archive: extdata_handle,
                }),
                false => Err(()),
            }
        }
    }

    pub fn read_file(&self, path: &str) -> Vec<u8> {
        const BATCH_SIZE: u32 = 1024;

        let mut file = Vec::<u8>::new();

        let mut path: Vec<u16> = path.encode_utf16().collect();
        path.push(0);

        unsafe {
            let mut handle: Handle = mem::zeroed();
            handle_error!(FSUSER_OpenFile(
                &mut handle as *mut _,
                self.archive,
                fsMakePath(PATH_UTF16, path.as_ptr() as *const c_void),
                OpenFlags::Read as u32,
                FileAttributes {
                    is_directory: false,
                    is_hidden: false,
                    is_archive: false,
                    readonly: true
                }
                .into()
            ));

            let mut read: u32 = mem::zeroed();
            let mut buffer: [u8; BATCH_SIZE as usize] = mem::zeroed();
            let mut offset: u64 = 0;

            loop {
                handle_error!(FSFILE_Read(
                    handle,
                    &mut read as *mut _,
                    offset,
                    &mut buffer as *mut _ as *mut c_void,
                    BATCH_SIZE
                ));
                offset += read as u64;
                for i in 0..read {
                    file.push(buffer[i as usize]);
                }
                if read < BATCH_SIZE {
                    break;
                }
            }

            handle_error!(FSFILE_Close(handle));
        }

        file
    }

    pub fn write_file(&self, path: &str, data: &[u8]) {
        unsafe {
            let mut handle: Handle = mem::zeroed();
            let mut path: Vec<u16> = path.encode_utf16().collect();
            path.push(0);

            let path = fsMakePath(PATH_UTF16, path.as_ptr() as *const c_void);

            handle_error!(FSUSER_DeleteFile(self.archive, path));

            handle_error!(FSUSER_CreateFile(self.archive, path, 0, data.len() as u64));

            handle_error!(FSUSER_OpenFile(
                &mut handle as *mut _,
                self.archive,
                path,
                OpenFlags::Write as u32,
                0
            ));

            let mut written: u32 = 0;

            handle_error!(FSFILE_Write(
                handle,
                &mut written as *mut _,
                0,
                data.as_ptr() as *const _,
                data.len() as u32,
                1
            ));

            handle_error!(FSFILE_Close(handle));
        }
    }

    pub fn read_manage(&self) -> Vec<u8> {
        self.read_file("/letter/manage.bin")
    }

    pub fn read<T: BPK1File>(&self) -> impl Iterator<Item = (FS_DirectoryEntry, String, T)> {
        self.list_dir("/letter")
            // I think I've read somewhere that it does this?
            // Notes are distributed among folders, so 0000, 0001...
            // Don't have that many notes to prove it, but better safe then sorry
            .filter(|(_path, dir)| is_letter_folder(string_from_filename(&dir.name)))
            .flat_map(move |(_path, dir)| {
                let directory = format!("/letter/{}", string_from_filename(&dir.name));
                self.list_dir(&directory)
            })
            .map(move |(path, entry)| {
                let file_name = string_from_filename(&entry.name);
                let file_path = format!("{}/{}", path, file_name);
                let file = self.read_file(&file_path);
                let letter = T::new_from_bpk1_bytes(&file).unwrap();
                (entry, file_path, letter)
            })
    }

    fn list_dir(&self, path: &str) -> DirectoryIterator {
        unsafe {
            let mut handle: Handle = mem::zeroed();
            let mut path_utf16: Vec<u16> = path.encode_utf16().collect();
            path_utf16.push(0); // NULL terminator
            handle_error!(FSUSER_OpenDirectory(
                &mut handle as *mut _,
                self.archive,
                fsMakePath(PATH_UTF16, path_utf16.as_ptr() as *const c_void),
            ));
            DirectoryIterator { path: path.to_string(), handle }
        }
    }
}

impl Drop for ExtdataArchive {
    fn drop(&mut self) {
        unsafe {
            handle_error!(FSUSER_CloseArchive(self.archive));
        }
    }
}

fn is_letter_folder(path: String) -> bool {
    path.chars().take(4).all(|c| c.is_numeric())
}

fn string_from_filename(name: &[u16; 262]) -> String {
    String::from_utf16(name)
        .unwrap()
        .split_terminator('\0')
        .take(1)
        .collect()
}

pub struct DirectoryIterator {
    path: String,
    handle: Handle,
}

impl Iterator for DirectoryIterator {
    type Item = (String, FS_DirectoryEntry);

    fn next(&mut self) -> Option<Self::Item> {
        let mut read: u32 = 0;
        let entry = unsafe {
            let mut entry: FS_DirectoryEntry = mem::zeroed();
            handle_error!(FSDIR_Read(
                self.handle,
                &mut read as *mut _,
                1,
                &mut entry as *mut _,
            ));
            entry
        };
        if read == 1 {
            Some((self.path.to_string(), entry))
        } else {
            None
        }
    }
}

impl Drop for DirectoryIterator {
    fn drop(&mut self) {
        unsafe {
            handle_error!(FSDIR_Close(self.handle));
        }
    }
}

#[repr(u32)]
enum OpenFlags {
    Read = 1,
    Write = 2,
    Create = 4,
}

#[repr(packed)]
struct FileAttributes {
    is_directory: bool,
    is_hidden: bool,
    is_archive: bool,
    readonly: bool,
}

impl From<FileAttributes> for u32 {
    fn from(val: FileAttributes) -> Self {
        unsafe { mem::transmute(val) }
    }
}
