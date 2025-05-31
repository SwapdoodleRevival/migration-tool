/*
 * We found ctru_sys to be a bit unreliable, so we're rawdogging it ourselves!
 */

use ctru_sys::{FriendInfo, FriendKey, Handle};
use libdoodle::mii_data::MiiData;
use std::{collections::HashMap, mem};

pub type MiiMap = HashMap<u32, MiiData>;

const FRIEND_LIST_SIZE: u32 = 100; // max number of friends is 100

pub fn load_friend_list() -> MiiMap {
    let mut friend_map = HashMap::new();

    unsafe {
        let mut frd_handle: Handle = 0;
        _ = ctru_sys::srvGetServiceHandle(&mut frd_handle as *mut _, c"frd:a".as_ptr());
        let (friend_keys, friends, num_friends) = get_friend_info(frd_handle);
        let me = get_my_info(frd_handle);
        _ = ctru_sys::svcCloseHandle(frd_handle);

        friend_map.insert(me.0, me.1);

        for i in 0..num_friends {
            let pid: u32 = friend_keys[i as usize].principalId;
            let mii_bytes: [u8; 0x5C] = friends[i as usize]._bindgen_opaque_blob[128..220]
                .try_into()
                .unwrap(); // Safe: known size

            if let Ok(mii) = MiiData::from_bytes(mii_bytes) {
                friend_map.insert(pid, mii);
            }
        }
    }

    friend_map
}

unsafe fn get_friend_info(handle: Handle) -> ([FriendKey; 100], [FriendInfo; 100], u32) {
    unsafe {
        let mut num_friends = 0u32;
        let mut friend_keys: [FriendKey; 100] = mem::zeroed();
        let mut friend_info: [FriendInfo; 100] = mem::zeroed();

        let cmdbuf = ctru_sys::getThreadCommandBuffer();
        *cmdbuf = 0x110080;
        *cmdbuf.wrapping_add(1) = 0x0;
        // offset 0 = take all friends
        *cmdbuf.wrapping_add(2) = FRIEND_LIST_SIZE;
        // max number of friends is 100
        *cmdbuf.wrapping_add(64) = (FRIEND_LIST_SIZE << 18) | 2;
        *cmdbuf.wrapping_add(65) = &mut friend_keys[0] as *mut _ as u32;

        _ = ctru_sys::svcSendSyncRequest(handle);

        if *cmdbuf.wrapping_add(1) != 0 {
            panic!("Something went wrong")
        }

        num_friends = *cmdbuf.wrapping_add(2);

        let cmdbuf = ctru_sys::getThreadCommandBuffer();
        *cmdbuf = 0x1A00C4;
        *cmdbuf.wrapping_add(1) = num_friends;
        *cmdbuf.wrapping_add(2) = 0;
        *cmdbuf.wrapping_add(3) = 0;
        *cmdbuf.wrapping_add(4) =
            (num_friends * mem::size_of::<FriendKey>() as u32) << 14 | ((0 & 0xF) << 10) | 0x2;
        *cmdbuf.wrapping_add(5) = &friend_keys[0] as *const _ as u32;
        *cmdbuf.wrapping_add(6) =
            (num_friends * mem::size_of::<FriendInfo>() as u32) << 4 | 0x8 | 0b100;
        *cmdbuf.wrapping_add(7) = &mut friend_info[0] as *mut _ as u32;

        _ = ctru_sys::svcSendSyncRequest(handle);

        if *cmdbuf.wrapping_add(1) != 0 {
            panic!("Something went wrong")
        }

        (friend_keys.clone(), friend_info, num_friends)
    }
}

unsafe fn get_my_info(handle: Handle) -> (u32, MiiData) {
    unsafe {
        let cmdbuf = ctru_sys::getThreadCommandBuffer();
        *cmdbuf = 0x00050000;
        _ = ctru_sys::svcSendSyncRequest(handle);
        let pid: u32 = *cmdbuf.add(2);

        if *cmdbuf.wrapping_add(1) != 0 {
            panic!("Something went wrong")
        }

        let cmdbuf = ctru_sys::getThreadCommandBuffer();
        *cmdbuf = 0x000A0000;
        _ = ctru_sys::svcSendSyncRequest(handle);

        if *cmdbuf.wrapping_add(1) != 0 {
            panic!("Something went wrong")
        }

        let mut mii: [u8; 0x5C] = mem::zeroed();
        let mut idx = 0usize;
        for i in 2..25 {
            for v in (*cmdbuf.wrapping_add(i)).to_le_bytes() {
                mii[idx] = v;
                idx += 1;
            }
        }

        (pid, MiiData::from_bytes(mii).unwrap())
    }
}
