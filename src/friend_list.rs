/*
 * We found ctru_sys to be a bit unreliable, so we're rawdogging it ourselves!
 */

use ctru_sys::{FriendInfo, FriendKey, Handle};
use libdoodle::blocks::miistd1::MiiData;
use std::{collections::HashMap, mem};

use crate::error::panic_if_failed;

pub type MiiMap = HashMap<u32, MiiData>;

const FRIEND_LIST_SIZE: u32 = 100; // max number of friends is 100

pub fn load_friend_list() -> MiiMap {
    let mut friend_map = HashMap::new();

    unsafe {
        let mut frd_handle: Handle = 0;
        _ = ctru_sys::srvGetServiceHandle(&mut frd_handle as *mut _, c"frd:a".as_ptr());
        get_friend_info(&mut friend_map, frd_handle);
        get_my_info(&mut friend_map, frd_handle);
        _ = ctru_sys::svcCloseHandle(frd_handle);
    }

    friend_map
}

unsafe fn get_friend_info(friend_map: &mut MiiMap, handle: Handle) {
    unsafe {
        let mut friend_keys: [FriendKey; 100] = mem::zeroed();
        let mut friend_info: [FriendInfo; 100] = mem::zeroed();

        let cmdbuf = ctru_sys::getThreadCommandBuffer();
        *cmdbuf = 0x110080;
        // offset 0 = take all friends
        *cmdbuf.wrapping_add(1) = 0x0;
        // max number of friends is 100
        *cmdbuf.wrapping_add(2) = FRIEND_LIST_SIZE;
        *cmdbuf.wrapping_add(64) = (FRIEND_LIST_SIZE << 18) | 2;
        *cmdbuf.wrapping_add(65) = &mut friend_keys[0] as *mut _ as u32;

        panic_if_failed!(ctru_sys::svcSendSyncRequest(handle));
        panic_if_failed!(*cmdbuf.wrapping_add(1));

        let num_friends = *cmdbuf.wrapping_add(2);

        let cmdbuf = ctru_sys::getThreadCommandBuffer();
        *cmdbuf = 0x1A00C4;
        *cmdbuf.wrapping_add(1) = num_friends;
        *cmdbuf.wrapping_add(2) = 1; // Mask non-ascii characters
        *cmdbuf.wrapping_add(3) = 0;
        *cmdbuf.wrapping_add(4) = ((num_friends * mem::size_of::<FriendKey>() as u32) << 14) | 0x2;
        *cmdbuf.wrapping_add(5) = &friend_keys[0] as *const _ as u32;
        *cmdbuf.wrapping_add(6) =
            (num_friends * mem::size_of::<FriendInfo>() as u32) << 4 | 0x8 | 0b100;
        *cmdbuf.wrapping_add(7) = &mut friend_info[0] as *mut _ as u32;

        panic_if_failed!(ctru_sys::svcSendSyncRequest(handle));
        panic_if_failed!(*cmdbuf.wrapping_add(1));

        for i in 0..num_friends {
            let pid: u32 = friend_keys[i as usize].principalId;
            let mii_bytes: [u8; 0x5C] =
                // There seems to be a bug in how FriendInfo is deserialized.
                // The Mii data is simply incorrect, the version should always be 3, but it isn't
                // Here I'm recreating the _bindgen_opaque_blob approach we had in earlier versions
                mem::transmute::<_, [u8; mem::size_of::<FriendInfo>()]>(friend_info[i as usize])
                    [128..220]
                    .try_into()
                    .unwrap();

            match MiiData::from_bytes(mii_bytes) {
                Ok(mii) => _ = friend_map.insert(pid, mii),
                Err(e) => println!("{:#?}", e),
            }
        }
    }
}

unsafe fn get_my_info(friend_map: &mut MiiMap, handle: Handle) {
    unsafe {
        let cmdbuf = ctru_sys::getThreadCommandBuffer();
        *cmdbuf = 0x00050000;
        panic_if_failed!(ctru_sys::svcSendSyncRequest(handle));
        panic_if_failed!(*cmdbuf.wrapping_add(1));
        let pid: u32 = *cmdbuf.add(2);

        let cmdbuf = ctru_sys::getThreadCommandBuffer();
        *cmdbuf = 0x000A0000;
        panic_if_failed!(ctru_sys::svcSendSyncRequest(handle));

        let mut mii: [u8; 0x5C] = mem::zeroed();
        let mut idx = 0usize;
        for i in 2..25 {
            for v in (*cmdbuf.wrapping_add(i)).to_le_bytes() {
                mii[idx] = v;
                idx += 1;
            }
        }
        panic_if_failed!(*cmdbuf.wrapping_add(1));

        friend_map.insert(pid, MiiData::from_bytes(mii).unwrap());
    }
}
