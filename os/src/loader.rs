extern "C" {
    fn _num_apps();
}

pub(crate) fn get_num_apps() -> usize {
    unsafe { (_num_apps as *const usize).read_volatile() }
}

pub(crate) fn get_app_data(app_id: usize) -> &'static [u8] {
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
