macro_rules! panic_if_failed {
    ($res: expr) => {
        let res = $res;
        if R_FAILED(res) {
            panic!("Error {res}");
        }
    };
}

pub(crate) use panic_if_failed;
