#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use libc;
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
            size: mem::size_of::<Self>() as i32,
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
            size: mem::size_of::<Self>() as i32,
            index: cam_id,
            hdcam: ptr::null_mut::<tag_dcam>(),
        }
    }
}

impl DCAMDEV_STRING {
    /// build a new `DCAMDEV_STRING` to query `istring` into `textbuf`
    pub fn new(istring: i32, textbuf: *mut raw::c_char) -> DCAMDEV_STRING {
        DCAMDEV_STRING {
            size: mem::size_of::<Self>() as i32,
            iString: istring,
            text: textbuf,
            textbytes: 256,
        }
    }
}

impl DCAMBUF_ATTACH {
    /// Build a new `DCAMBUF_ATTACH` struct
    pub fn new(buffer: *mut *mut libc::c_void, nframes: usize) -> DCAMBUF_ATTACH {
        DCAMBUF_ATTACH {
            size: mem::size_of::<Self>() as i32,
            iKind: DCAM_ATTACHKIND_DCAMBUF_ATTACHKIND_FRAME,
            buffer,
            buffercount: nframes as int32,
        }
    }
}

impl DCAMCAP_TRANSFERINFO {
    pub fn new() -> DCAMCAP_TRANSFERINFO {
        DCAMCAP_TRANSFERINFO {
            size: mem::size_of::<Self>() as i32,
            iKind: DCAMCAP_TRANSFERKIND_DCAMCAP_TRANSFERKIND_FRAME,
            nNewestFrameIndex: -1,
            nFrameCount: -1,
        }
    }
}

impl DCAMWAIT_OPEN {
    pub fn new(camera_handle: HDCAM) -> DCAMWAIT_OPEN {
        DCAMWAIT_OPEN {
            size: mem::size_of::<Self>() as int32,
            supportevent: DCAMWAIT_EVENT_DCAMWAIT_CAPEVENT_FRAMEREADY,
            hwait: ptr::null_mut::<DCAMWAIT>(),
            hdcam: camera_handle,
        }
    }
}

impl DCAMWAIT_START {
    pub fn new() -> DCAMWAIT_START {
        DCAMWAIT_START {
            size: mem::size_of::<Self>() as int32,
            eventhappened: 0,
            eventmask: DCAMWAIT_EVENT_DCAMWAIT_CAPEVENT_FRAMEREADY,
            timeout: DCAMWAIT_TIMEOUT_DCAMWAIT_TIMEOUT_INFINITE,
        }
    }
}
