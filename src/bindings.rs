#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::mem;
use std::os::raw;
use std::ptr;

impl DCAM_GUID {
    /// Make a new `DCAM_GUID` struct initialized with zeros
    pub fn new() -> DCAM_GUID {
        DCAM_GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        }
    }
}

impl DCAMAPI_INIT {
    pub fn new(dcam_guid: *const DCAM_GUID) -> DCAMAPI_INIT {
        DCAMAPI_INIT {
            size: mem::size_of::<DCAMAPI_INIT>() as i32,
            iDeviceCount: 0,
            reserved: 0,
            initoptionbytes: 0,
            initoption: ptr::null(),
            guid: dcam_guid,
        }
    }
}

impl DCAMDEV_OPEN {
    /// create a new instance to request the camera with id `cam_id`
    pub fn new(cam_id: i32) -> DCAMDEV_OPEN {
        DCAMDEV_OPEN {
            size: mem::size_of::<DCAMDEV_OPEN>() as i32,
            index: cam_id,
            hdcam: ptr::null_mut::<tag_dcam>(),
        }
    }
}

impl DCAMDEV_STRING {
    /// build a new `DCAMDEV_STRING` to query `istring` into `textbuf`
    pub fn new(istring: i32, textbuf: *mut raw::c_char) -> DCAMDEV_STRING {
        DCAMDEV_STRING {
            size: mem::size_of::<DCAMDEV_STRING>() as i32,
            iString: istring,
            text: textbuf,
            textbytes: 256,
        }
    }
}
