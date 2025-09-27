use std::{mem, os::raw::c_void};

use ctru_sys::{
    FS_Archive, FS_Path, FSFILE_Close, FSFILE_Read, FSFILE_Write, FSUSER_CloseArchive,
    FSUSER_CreateFile, FSUSER_DeleteFile, FSUSER_OpenArchive, FSUSER_OpenFile, Handle,
    MEDIATYPE_SD, PATH_BINARY, PATH_UTF16, R_FAILED, R_SUCCEEDED, fsMakePath,
};

use crate::error::panic_if_failed;

pub enum SwapdoodleRegion {
    EU,
    US,
    JP,
}

pub struct ExtdataArchive {
    pub region: SwapdoodleRegion,
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
                    region,
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
            panic_if_failed!(FSUSER_OpenFile(
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
                panic_if_failed!(FSFILE_Read(
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

            panic_if_failed!(FSFILE_Close(handle));
        }

        file
    }

    pub fn write_file(&self, path: &str, data: &[u8]) {
        unsafe {
            let mut handle: Handle = mem::zeroed();
            let mut path: Vec<u16> = path.encode_utf16().collect();
            path.push(0);

            let path = fsMakePath(PATH_UTF16, path.as_ptr() as *const c_void);

            panic_if_failed!(FSUSER_DeleteFile(self.archive, path));

            panic_if_failed!(FSUSER_CreateFile(self.archive, path, 0, data.len() as u64));

            panic_if_failed!(FSUSER_OpenFile(
                &mut handle as *mut _,
                self.archive,
                path,
                OpenFlags::Write as u32,
                0
            ));

            let mut written: u32 = 0;

            panic_if_failed!(FSFILE_Write(
                handle,
                &mut written as *mut _,
                0,
                data.as_ptr() as *const _,
                data.len() as u32,
                1
            ));

            panic_if_failed!(FSFILE_Close(handle));
        }
    }

    fn filename_from_key(key: u32) -> String {
        let folder = key / 200;
        format!("/letter/{:04}/lt{:04}.bin", folder, key)
    }

    pub fn read_letter_index(&self, key: u32) -> Vec<u8> {
        let filename = ExtdataArchive::filename_from_key(key);
        println!("Reading {}...", filename);
        self.read_file(&filename)
    }

    pub fn write_letter_index(&self, key: u32, data: &[u8]) {
        let filename = ExtdataArchive::filename_from_key(key);
        println!("Writing {}...", filename);
        self.write_file(&filename, data)
    }

    pub fn read_manage(&self) -> Vec<u8> {
        self.read_file("/letter/manage.bin")
    }
}

impl Drop for ExtdataArchive {
    fn drop(&mut self) {
        unsafe {
            panic_if_failed!(FSUSER_CloseArchive(self.archive));
        }
    }
}

#[repr(u32)]
#[allow(unused)]
enum OpenFlags {
    Read = 1,
    Write = 2,
    Create = 4,
}

#[repr(C, packed)]
#[allow(unused)]
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
