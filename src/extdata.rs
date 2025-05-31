use std::{mem, os::raw::c_void, time::Instant, u32};

use ctru_sys::{
    self, FS_Archive, FS_DirectoryEntry, FS_MediaType, FS_Path, FSDIR_Close, FSDIR_Control,
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

    list_dir(extdata_handle, "/letter/0000\0")
        .into_iter()
        .map(move |v| {
            let filename = String::from_utf16(&v.name).unwrap();
            let filename = format!("/letter/0000/{filename}\0");
            let file = read_file(extdata_handle, &filename);
            let letter = Letter::new_from_bpk1_bytes(&file).unwrap();
            (v, filename, letter)
        })
}

struct DirectoryIterator {
    handle: Handle,
}

impl Iterator for DirectoryIterator {
    type Item = FS_DirectoryEntry;

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
        if read == 1 { Some(entry) } else { None }
    }
}

impl Drop for DirectoryIterator {
    fn drop(&mut self) {
        unsafe {
            handle_error!(FSDIR_Close(self.handle));
        }
    }
}

fn list_dir(archive: FS_Archive, path: &str) -> DirectoryIterator {
    unsafe {
        let mut handle: Handle = mem::zeroed();
        let path: Vec<u16> = path.encode_utf16().collect();
        handle_error!(FSUSER_OpenDirectory(
            &mut handle as *mut _,
            archive,
            fsMakePath(PATH_UTF16, path.as_ptr() as *const c_void),
        ));
        DirectoryIterator { handle }
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
        let path: Vec<u16> = path.encode_utf16().collect();
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
