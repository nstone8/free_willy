use image::{ImageBuffer, Luma};
use libc;
use std::ffi::CStr;
use std::ops::Index;
use std::ops::IndexMut;
use std::os::raw;
use std::ptr;
use std::sync::mpsc::{channel, sync_channel, Receiver, RecvError, Sender, TryRecvError};
use std::thread::{self, JoinHandle};

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
    pub fn open_cam<T: Camera>(&self, cam_id: i32) -> Result<T, i32> {
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
        Ok(T::new(dco.hdcam))
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
pub struct C11440_22CU {
    handle: bindings::HDCAM,
}

#[allow(drop_bounds)] //I want to make sure all cameras implement Drop so we don't end up with dangling camera handles
pub trait Camera: Drop {
    //type Cam: Camera;
    /// Get a HDCAM handle from the API
    fn new(handle: bindings::HDCAM) -> Self;
    /// Get the current exposure time, on error this returns an `Err(DCAM_ERROR)`
    fn get_exposure(&self) -> Result<f64, i32>;
    /// get the current image width, on error this returns an `Err(DCAM_ERROR)`
    fn get_image_width(&self) -> Result<i32, i32>;
    /// get the current image height, on error this returns an `Err(DCAM_ERROR)`
    fn get_image_height(&self) -> Result<i32, i32>;
    /// get the number of bytes per frame with current settings
    fn get_framebytes(&self) -> Result<usize, i32>;
    /// Get the current image resolution `[w,h]`, on error this returns an `Err(DCAM_ERROR)`
    fn get_resolution(&self) -> Result<[i32; 2], i32> {
        let h = self.get_image_height()?;
        let w = self.get_image_width()?;
        Ok([w, h])
    }
    /// Get the api handle for this camera
    fn handle(&self) -> bindings::HDCAM;
    /// use the 'raw' dcamdev_getstring() function to get camera info.
    /// currently the API will copy the string into a buffer with a fixed length of 256 bytes
    fn dcamdev_getstring(&self, istring: i32) -> Result<String, String> {
        // make a buffer to store the result
        // I set the size of the buffer to 256 in the implementation of DCAMDEV_STRING::new
        let mut carray: [raw::c_char; 256] = [0; 256];
        let mut dcds = bindings::DCAMDEV_STRING::new(istring, carray.as_mut_ptr());
        unsafe {
            let err = bindings::dcamdev_getstring(self.handle(), &mut dcds);
            if err != 1 {
                return Err(format!(
                    "call to dcamdev_getstring failed with code {}",
                    err
                ));
            }
        }
        //Convert to a String
        let cstr = unsafe { CStr::from_ptr(carray.as_ptr()) };
        let string_result = cstr.to_str();
        if let Ok(s) = string_result {
            return Ok(String::from(s));
        } else {
            return Err(String::from("invalid UTF8"));
        }
    }
    /// get the camera model
    fn model(&self) -> Result<String, String> {
        self.dcamdev_getstring(bindings::DCAM_IDSTR_DCAM_IDSTR_MODEL)
    }
    /// get the DCAM API version supported by the camera
    fn api_version(&self) -> Result<String, String> {
        self.dcamdev_getstring(bindings::DCAM_IDSTR_DCAM_IDSTR_DCAMAPIVERSION)
    }
    /// get the camera's serial number
    fn serial_number(&self) -> Result<String, String> {
        self.dcamdev_getstring(bindings::DCAM_IDSTR_DCAM_IDSTR_CAMERAID)
    }
    /// call the API dcamprop_getvalue to get the property associated with `i_prop`
    fn dcamprop_getvalue(&self, i_prop: bindings::int32) -> Result<f64, i32> {
        let mut val: f64 = 0.0;
        let err = unsafe { bindings::dcamprop_getvalue(self.handle(), i_prop, &mut val) };
        match err {
            1 => Ok(val),
            e => Err(e),
        }
    }
    /// call the API dcamprop_getvalue to set the property associated with `i_prop` to `f_value`
    fn dcamprop_setvalue(&self, i_prop: bindings::int32, f_value: f64) -> Result<(), i32> {
        let err = unsafe { bindings::dcamprop_setvalue(self.handle(), i_prop, f_value) };
        match err {
            1 => Ok(()),
            e => Err(e),
        }
    }
    /// allocate and attach a `FrameBuffer` capable of holding `num_frames`
    fn attach_buffer(&self, num_frames: usize) -> Result<FrameBuffer, i32> {
        let frame_size = self.get_framebytes()?;
        FrameBuffer::attach(self.handle(), frame_size, num_frames)
    }
}

impl Camera for C11440_22CU {
    //type Cam = C11440_22CU;
    fn new(handle: bindings::HDCAM) -> Self {
        C11440_22CU { handle }
    }
    fn handle(&self) -> bindings::HDCAM {
        self.handle
    }
    fn get_exposure(&self) -> Result<f64, i32> {
        self.dcamprop_getvalue(bindings::_DCAMIDPROP_DCAM_IDPROP_EXPOSURETIME)
    }
    fn get_image_width(&self) -> Result<i32, i32> {
        match self.dcamprop_getvalue(bindings::_DCAMIDPROP_DCAM_IDPROP_IMAGE_WIDTH) {
            Ok(f) => Ok(f as i32),
            Err(e) => Err(e),
        }
    }

    fn get_image_height(&self) -> Result<i32, i32> {
        match self.dcamprop_getvalue(bindings::_DCAMIDPROP_DCAM_IDPROP_IMAGE_HEIGHT) {
            Ok(f) => Ok(f as i32),
            Err(e) => Err(e),
        }
    }
    fn get_framebytes(&self) -> Result<usize, i32> {
        match self.dcamprop_getvalue(bindings::_DCAMIDPROP_DCAM_IDPROP_BUFFER_FRAMEBYTES) {
            Ok(f) => Ok(f as usize),
            Err(e) => Err(e),
        }
    }
}

/// Automatically release camera when our handle is dropped
impl Drop for C11440_22CU {
    fn drop(&mut self) {
        unsafe {
            bindings::dcamdev_close(self.handle);
        }
    }
}

/// A struct representing a framebuffer the camera can copy images into
/// Each frame is `frame_size` bytes in size and there are `num_frames` frames allocated
pub struct FrameBuffer {
    camera_handle: bindings::HDCAM,
    frame_size: usize,
    num_frames: usize,
    buffer: Vec<u16>,
}

///Make it so we can pull one frame's worth of data by index
impl Index<usize> for FrameBuffer {
    type Output = [u16];
    fn index(&self, index: usize) -> &[u16] {
        assert!(index < self.num_frames);
        &self.buffer[index * self.frame_size / 2..(index + 1) * self.frame_size / 2]
    }
}

impl IndexMut<usize> for FrameBuffer {
    fn index_mut(&mut self, index: usize) -> &mut [u16] {
        assert!(index < self.num_frames);
        &mut self.buffer[index * self.frame_size / 2..(index + 1) * self.frame_size / 2]
    }
}

impl FrameBuffer {
    /// Allocate memory for a buffer to hold image data and inform the API of the address
    pub fn attach(
        camera_handle: bindings::HDCAM,
        frame_size: usize,
        num_frames: usize,
    ) -> Result<FrameBuffer, i32> {
        let mut me = FrameBuffer {
            camera_handle: camera_handle,
            frame_size,
            num_frames,
            buffer: vec![0; (frame_size) / 2 * num_frames],
        };
        //we need to create an array of pointers to each frame
        let mut pvec: Vec<*mut libc::c_void> = (0..num_frames)
            .map(|i| me[i].as_mut_ptr() as *mut libc::c_void)
            .collect();
        let bufptr = pvec[0..num_frames].as_mut_ptr() as *mut *mut libc::c_void;
        let dcba = bindings::DCAMBUF_ATTACH::new(bufptr, num_frames);
        // attach the buffer to the API
        match unsafe { bindings::dcambuf_attach(camera_handle, &dcba) } {
            1 => Ok(me),
            e => Err(e),
        }
    }
    /// call the API's dcamcap_transferinfo function to recieve a tuple containing
    /// `(most_recent_frame, total_frames_captured)`
    fn dcamcap_transferinfo(&self) -> Result<(usize, i32), i32> {
        let mut ti = bindings::DCAMCAP_TRANSFERINFO::new();
        match unsafe { bindings::dcamcap_transferinfo(self.camera_handle, &mut ti) } {
            1 => Ok((ti.nNewestFrameIndex as usize, ti.nFrameCount)),
            e => Err(e),
        }
    }
    /// get the index of the most recent frame captured in the buffer
    fn most_recent_frame_index(&self) -> Result<usize, i32> {
        match self.dcamcap_transferinfo() {
            Ok(t) => Ok(t.0),
            Err(e) => Err(e),
        }
    }
    ///get a copy of the most recently captured frame
    fn copy_most_recent_frame(&self) -> Result<Vec<u16>, i32> {
        match self.most_recent_frame_index() {
            Ok(i) => Ok(self[i].to_vec()),
            Err(e) => Err(e),
        }
    }
    /// get an api wait handle
    fn get_wait_handle(&self) -> Result<bindings::HDCAMWAIT, i32> {
        let mut dcwo = bindings::DCAMWAIT_OPEN::new(self.camera_handle);
        match unsafe { bindings::dcamwait_open(&mut dcwo) } {
            1 => Ok(dcwo.hwait),
            e => Err(e),
        }
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            bindings::dcambuf_release(
                self.camera_handle,
                bindings::DCAM_ATTACHKIND_DCAMBUF_ATTACHKIND_FRAME,
            );
        }
    }
}

///Struct for representing a source of frames from a C11440_22CU camera. Can call stream() to get a frame stream.
pub struct C11440_22CUSource {
    camid: i32,
}

///Struct for representing a stream of frames. can call recv to grab frames, stop to stop
pub struct DcamStream {
    ///channel we are recieving streams on
    frame_rx: Receiver<ImageBuffer<Luma<u16>, Vec<u16>>>,
    ///channel we use to kill the capture thread
    control_tx: Sender<()>,
    thread_handle: JoinHandle<()>,
}

impl C11440_22CUSource {
    /// Create a new `DcamSource` for pulling frames off of the camera with API index `camid`
    pub fn new(camid: i32) -> C11440_22CUSource {
        C11440_22CUSource { camid }
    }
    ///Stream from a C11440_22CU. `bufsize` is the size of the image buffer
    ///as well as the size of the buffer the frames are written to by the thread spawned here
    pub fn stream(&self, bufsize: usize) -> DcamStream {
        stream::<C11440_22CU>(self.camid, bufsize)
    }
}

///Stream from a camera with type `T` and API index `camid`. `bufsize` is the size of the image buffer
///as well as the size of the buffer the frames are written to by the thread spawned here.
fn stream<T: Camera>(camid: i32, bufsize: usize) -> DcamStream {
    //build our channels
    let (frame_tx, frame_rx) = sync_channel::<ImageBuffer<Luma<u16>, Vec<u16>>>(bufsize);
    let (control_tx, control_rx) = channel::<()>();
    //make a copy of our camid
    //spawn a thread that initializes the camera and starts shoving frames into frametx
    let thread_handle = thread::spawn(move || {
        let api = DcamAPI::connect().expect("couldn't communicate with API");
        let cam = api
            .open_cam::<T>(camid)
            .expect("Couldn't get camera handle");
        let framebuffer = cam.attach_buffer(bufsize).expect("couldn't attach buffer");
        //get our image size
        let imsize = cam.get_resolution().expect("couldn't get image resolution");
        //get a wait handle
        let mut dws = bindings::DCAMWAIT_START::new();
        let hwait = framebuffer
            .get_wait_handle()
            .expect("Couldn't get wait handle");
        //start capturing
        //pick up here, add logic for safely stopping the thread through controlrx, return the struct
        let err = unsafe {
            bindings::dcamcap_start(
                framebuffer.camera_handle,
                bindings::DCAMCAP_START_DCAMCAP_START_SEQUENCE,
            )
        };
        assert_eq!(1, err, "couldn't start acquisition");
        loop {
            //check to see if we've been asked to stop
            match control_rx.try_recv() {
                //No messages, channel still open so we continue
                Err(TryRecvError::Empty) => {}
                //All other options (channel closed, received a value) mean stop
                _ => break,
            }
            let err = unsafe {
                // Wait for the API to tell us about a new frame
                bindings::dcamwait_start(hwait, &mut dws)
            };
            assert_eq!(1, err);
            //grab the newest frame
            let new_frame_raw = framebuffer
                .copy_most_recent_frame()
                .expect("failed to copy frame");
            //convert to an image
            let new_frame = ImageBuffer::<Luma<u16>, Vec<u16>>::from_raw(
                imsize[0] as u32,
                imsize[1] as u32,
                new_frame_raw,
            )
            .expect("failed to convert to image");
            //send the new frame down the buffer
            match frame_tx.try_send(new_frame) {
                Ok(()) => {}
                Err(_) => panic!("buffer overflow"),
            }
        }
    });
    DcamStream {
        frame_rx,
        control_tx,
        thread_handle,
    }
}

impl DcamStream {
    /// Pull a frame off of the buffer
    pub fn recv(&self) -> Result<ImageBuffer<Luma<u16>, Vec<u16>>, RecvError> {
        self.frame_rx.recv()
    }
    /// Stop pulling frames, deallocate the buffer
    pub fn stop(self) {
        self.control_tx
            .send(())
            .expect("Couldn't communicate with frame grabber");
        self.thread_handle
            .join()
            .expect("Couldn't shut down frame grabber");
    }
}
