use std::ffi::CStr;
use std::os::raw;
use std::ptr;

pub mod bindings;

/// `struct` to represent an instance of the DCAM API
pub struct DcamAPI {
    /// `free_willy::bindings::DCAMAPI_INIT` struct used to initialize the API
    api_init: bindings::DCAMAPI_INIT,
}

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

    /// Get a handle to a camera
    pub fn open_cam(&self, cam_id: i32) -> Result<Camera, i32> {
        if cam_id > self.ncam() {
            eprintln!("camera index cannot be greater than (ncam - 1)");
            return Err(bindings::DCAMERR_DCAMERR_INVALIDCAMERA);
        }
        //make a new DCAMDEV_OPEN struct and try to open the camera
        let mut dco = bindings::DCAMDEV_OPEN::new(cam_id);
        unsafe {
            let err = bindings::dcamdev_open(&mut dco);
            if err != 1 {
                return Err(err);
            }
        }
        //if the open worked, the hdcam pointer should not be null
        assert!(!dco.hdcam.is_null(), "null camera pointer");
        Ok(Camera { handle: dco.hdcam })
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

/// `struct` to represent a camera
pub struct Camera {
    handle: bindings::HDCAM,
}

impl Camera {
    /// use the 'raw' dcamdev_getstring() function to get camera info.
	/// currently the API will copy the string into a buffer with a fixed length of 256 bytes
    fn dcamdev_getstring(&self, istring: i32) -> Result<String, i32> {
        // make a buffer to store the result
        // I set the size of the buffer to 256 in the implementation of DCAMDEV_STRING::new
        let mut carray: [raw::c_char; 256] = [0; 256];
        let mut dcds = bindings::DCAMDEV_STRING::new(istring, carray.as_mut_ptr());
        unsafe {
            let err = bindings::dcamdev_getstring(self.handle, &mut dcds);
            if err != 1 {
                return Err(err);
            }
        }
        //Convert to a String
        let cstr = unsafe { CStr::from_ptr(carray.as_ptr()) };
        let string_result = cstr.to_str();
        if let Ok(s) = string_result {
            return Ok(String::from(s));
        } else {
            eprintln!("invalid UTF8");
            return Err(bindings::DCAMERR_DCAMERR_INVALIDCAMERA);
        }
    }
    /// get the camera model
    pub fn model(&self) -> String {
        self.dcamdev_getstring(bindings::DCAM_IDSTR_DCAM_IDSTR_MODEL)
            .expect("couldn't read model")
    }
    /// get the DCAM API version supported by the camera
    pub fn api_version(&self) -> String {
        self.dcamdev_getstring(bindings::DCAM_IDSTR_DCAM_IDSTR_DCAMAPIVERSION)
            .expect("couldn't read api version")
    }
    /// get the camera's serial number
    pub fn serial_number(&self) -> String {
        self.dcamdev_getstring(bindings::DCAM_IDSTR_DCAM_IDSTR_CAMERAID)
            .expect("couldn't read api version")
    }
}

/// Automatically release camera when our handle is dropped
impl Drop for Camera {
    fn drop(&mut self) {
        unsafe {
            bindings::dcamdev_close(self.handle);
        }
    }
}
