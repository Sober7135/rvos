use alloc::vec::Vec;
use lazy_static::*;

extern "C" {
    fn _num_apps();
}

lazy_static! {
    static ref APP_NAMES: Vec<&'static str> = {
        let num_apps = get_num_apps();
        extern "C" {
            fn _app_names();
        }

        let mut start = _app_names as usize as *mut u8;
        let mut v = Vec::new();
        unsafe {
            for _ in 0..num_apps {
                let mut end = start;
                while end.read_volatile() != 0 {
                    end = end.add(1);
                }
                v.push(core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                    start,
                    end as usize - start as usize,
                )));
                start = end.add(1);
            }
        }
        v
    };
}

pub fn get_num_apps() -> usize {
    unsafe { (_num_apps as *const usize).read_volatile() }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    let num_app_ptr = _num_apps as *const usize;
    let num_apps = get_num_apps();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_apps + 1) };
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    }
}

pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
    let mut idx = None;
    for (i, &app_name) in APP_NAMES.iter().enumerate() {
        if name == app_name {
            idx = Some(i);
        }
    }
    Some(get_app_data(idx?))
}

pub fn init() {
    list_apps();
}

fn list_apps() {
    println!("=============================== APPS ===============================");
    for app in APP_NAMES.iter() {
        println!("{}", app);
    }
    println!("====================================================================");
}
