use std::{mem, os::raw::c_void, u32};

use ctru_sys::{
    self, FS_Archive, FS_DirectoryEntry, FS_MediaType, FS_Path, FSDIR_Close,
    FSDIR_Read, FSFILE_Close, FSFILE_Read, FSUSER_OpenArchive, FSUSER_OpenDirectory,
    FSUSER_OpenFile, Handle, MEDIATYPE_SD, PATH_BINARY, PATH_UTF16, R_FAILED, R_SUCCEEDED,
    fsMakePath,
};
use libdoodle::bpk1::{BPK1File, letter::Letter};

macro_rules! handle_error {
    ($res: expr) => {
        let res = $res;
        if R_FAILED(res) {
            panic!("Error {res}");
        }
    };
}

pub fn read() -> impl Iterator<Item = (FS_DirectoryEntry, String, Letter)> {
    // returns (file_path, vec<u8>)
    let extdata_handle: FS_Archive = open_title_extdata(MEDIATYPE_SD, 0x00040000001A2E00).unwrap();

    list_dir(extdata_handle, "/letter".to_string())
        .into_iter()
        // I think I've read somewhere that it does this?
        // Notes are distributed among folders, so 0000, 0001...
        // Don't have that many notes to prove it, but better safe then sorry
        .filter(|(_path, dir)| is_letter_folder(string_from_filename(&dir.name)))
        .flat_map(move |(_path, dir)| {
            let directory = format!("/letter/{}", string_from_filename(&dir.name));
            return list_dir(extdata_handle, directory);
        })
        .map(move |(path, entry)| {
            let file_name = string_from_filename(&entry.name);
            let file_path = format!("{}/{}", path, file_name);
            let file = read_file(extdata_handle, &file_path);
            let letter = Letter::new_from_bpk1_bytes(&file).unwrap();
            (entry, file_path, letter)
        })
}

fn is_letter_folder(path: String) -> bool {
    return path.chars().take(4).all(|c| c.is_numeric());
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
        if read == 1 { Some((self.path.clone(), entry)) } else { None }
    }
}

impl Drop for DirectoryIterator {
    fn drop(&mut self) {
        unsafe {
            handle_error!(FSDIR_Close(self.handle));
        }
    }
}

fn list_dir(archive: FS_Archive, path: String) -> DirectoryIterator {
    unsafe {
        let mut handle: Handle = mem::zeroed();
        let mut path_utf16: Vec<u16> = path.encode_utf16().collect();
        path_utf16.push(0); // NULL terminator
        handle_error!(FSUSER_OpenDirectory(
            &mut handle as *mut _,
            archive,
            fsMakePath(PATH_UTF16, path_utf16.as_ptr() as *const c_void),
        ));
        DirectoryIterator { path, handle }
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

impl Into<u32> for FileAttributes {
    fn into(self) -> u32 {
        unsafe { mem::transmute(self) }
    }
}

fn read_file(archive: FS_Archive, path: &str) -> Vec<u8> {
    const BATCH_SIZE: u32 = 1024;
    let mut file = Vec::<u8>::new();

    unsafe {
        let mut handle: Handle = mem::zeroed();
        let mut path: Vec<u16> = path.encode_utf16().collect();
        path.push(0);
        handle_error!(FSUSER_OpenFile(
            &mut handle as *mut _,
            archive,
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

fn open_title_extdata(media_type: FS_MediaType, title_id: u64) -> Option<FS_Archive> {
    unsafe {
        let mut extdata_handle: FS_Archive = mem::zeroed();

        let extdata = (title_id as u32) >> 8;

        let path: [u32; 3] = [media_type.into(), extdata, 0];

        R_SUCCEEDED(FSUSER_OpenArchive(
            &mut extdata_handle as *mut _,
            0x00000006, // ARCHIVE_EXTDATA
            FS_Path {
                type_: PATH_BINARY,
                size: 12,
                data: &path as *const _ as *const c_void,
            },
        ))
        .then_some(extdata_handle)
    }
}
