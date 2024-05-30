use free_willy::Camera;
fn main() {
    //let's try initializing the API and see what happens
    //let mut guid = bindings::DCAM_GUID::new();
    let api_result = free_willy::DcamAPI::connect();
    match api_result {
        Ok(api) => {
            println!("Connected, detecting {} camera", api.ncam());
            match api.open_cam::<free_willy::C11440_22CU>(0) {
                Ok(cam) => {
                    println!("got camera handle");
                    println!("camera model: {}", cam.model().unwrap());
                    println!("camera SN#: {}", cam.serial_number().unwrap());
                    println!("camera api version {}", cam.api_version().unwrap());
                }
                Err(e) => println!("failed to open camera, error: {}", e),
            }
        }
        Err(e) => println!("Failed, error code: {}", e),
    }
}
