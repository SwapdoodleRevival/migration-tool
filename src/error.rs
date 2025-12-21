macro_rules! panic_if_failed {
    ($res: expr) => {
        let res = $res;
        if ::ctru_sys::R_FAILED(res as i32) {
            panic!("Error {res}");
        }
    };
}

pub(crate) use panic_if_failed;
