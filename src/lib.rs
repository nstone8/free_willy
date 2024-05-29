use std::ptr;

pub mod bindings;

/// `struct` to represent our instance of the DCAM API
pub struct DcamAPI {
    /// `free_willy::bindings::DCAMAPI_INIT` struct used to initialize the API
    api_init: bindings::DCAMAPI_INIT,
}

/// `struct` to represent a camera
pub struct Camera {}

impl DcamAPI {
    /// Connect to the DCAM API, if we failed, the error code will be returned wrapped in the `Err(i32)`
    pub fn connect() -> Result<DcamAPI, i32> {
        let mut api = bindings::DCAMAPI_INIT::new(ptr::null());
        let err: i32;
        unsafe {
            err = bindings::dcamapi_init(&mut api);
        }
        if err == 1 {
            Ok(DcamAPI { api_init: api })
        } else {
            Err(err)
        }
    }
    /// get the number of connected cameras
    pub fn ncam(&self) -> i32 {
        self.api_init.iDeviceCount
    }
}

/// Automatically release the API when our API handle is dropped
impl Drop for DcamAPI {
    fn drop(&mut self) {
        unsafe {
            bindings::dcamapi_uninit();
        }
    }
}
