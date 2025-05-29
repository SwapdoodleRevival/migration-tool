/*Result ListFriends(FriendKey *friendKeyList, u32 *num, u32 offset, u32 size)
{
    Result ret = 0;
    u32 *cmdbuf = getThreadCommandBuffer();

    cmdbuf[0] = IPC_MakeHeader(0x11, 2, 0); // 0x110080
    cmdbuf[1] = offset;
    cmdbuf[2] = size;
    cmdbuf[64] = (size << 18) | 2;
    cmdbuf[65] = (u32)friendKeyList;

    if (R_FAILED(ret = svcSendSyncRequest(frdHandle)))
        return ret;

    *num = cmdbuf[2];

    return (Result)cmdbuf[1];
}*/

use core::num;
use ctru_sys::{FriendInfo, FriendKey, Handle};
use libdoodle::mii_data::{MiiData, MiiDataBytes};
use std::{collections::HashMap, mem, ops::Index};

const FRIEND_LIST_SIZE: u32 = 100; // max number of friends is 100
pub fn load_friend_list() -> HashMap<u32, MiiData> {
    let mut friend_map = HashMap::new();

    unsafe {
        let mut frd_handle: Handle = 0;
        _ = ctru_sys::srvGetServiceHandle(&mut frd_handle as *mut _, c"frd:a".as_ptr());
        let (friend_keys, friends, num_friends) = get_friend_info(frd_handle);
        _ = ctru_sys::svcCloseHandle(frd_handle);

        for i in 0..num_friends {
            let pid: u32 = friend_keys[i as usize].principalId;
            let localFriendCode: u64 = friend_keys[i as usize].localFriendCode;
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
        let mut friend_keys: [FriendKey; 100] = mem::transmute([1u8; 1600]);
        let mut friend_info: [FriendInfo; 100] = mem::transmute([1u8; 22400]); // mem::zeroed()

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
            IPC_Desc_StaticBuffer(num_friends * mem::size_of::<FriendKey>() as u32, 0);
        *cmdbuf.wrapping_add(5) = &friend_keys[0] as *const _ as u32;
        *cmdbuf.wrapping_add(6) =
            IPC_Desc_Buffer(num_friends * mem::size_of::<FriendInfo>() as u32);
        *cmdbuf.wrapping_add(7) = &mut friend_info[0] as *mut _ as u32;

        _ = ctru_sys::svcSendSyncRequest(handle);

        if *cmdbuf.wrapping_add(1) != 0 {
            panic!("Something went wrong")
        }

        (friend_keys.clone(), friend_info, num_friends)
    }
}

fn IPC_Desc_StaticBuffer(size: u32, buffer_id: u32) -> u32 {
    size << 14 | ((buffer_id & 0xF) << 10) | 0x2
}

fn IPC_Desc_Buffer(size: u32) -> u32 {
    (size << 4) | 0x8 | 0b100
}

/*
Result FRD_GetFriendInfo(FriendInfo *infos, const FriendKey *friendKeyList, u32 count, bool maskNonAscii, bool profanityFlag)
{
    Result ret = 0;
    u32 *cmdbuf = getThreadCommandBuffer();

    cmdbuf[0] = IPC_MakeHeader(0x1A,3,4); // 0x1A00C4
    cmdbuf[1] = count;
    cmdbuf[2] = (u32)maskNonAscii;
    cmdbuf[3] = (u32)profanityFlag;
    cmdbuf[4] = IPC_Desc_StaticBuffer(count * sizeof(FriendKey), 0);
    cmdbuf[5] = (u32)friendKeyList;
    cmdbuf[6] = IPC_Desc_Buffer(count * sizeof(FriendInfo), IPC_BUFFER_W);
    cmdbuf[7] = (u32)infos;

    if (R_FAILED(ret = svcSendSyncRequest(frdHandle))) return ret;

    return (Result)cmdbuf[1];
}
*/
