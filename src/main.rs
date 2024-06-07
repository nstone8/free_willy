use free_willy::viewer;
use free_willy::Camera;
use iced::{Application, Settings};
fn main() {
    //print camera info
    match free_willy::DcamAPI::connect() {
        Ok(api) => {
            println!("Connected, detecting {} camera", api.ncam());
            match api.open_cam::<free_willy::C11440_22CU>(0) {
                Ok(cam) => {
                    println!("got camera handle");
                    println!("camera model: {}", cam.model().unwrap());
                    println!("camera SN#: {}", cam.serial_number().unwrap());
                    println!("camera api version {}", cam.api_version().unwrap());
                    println!("exposure setting: {}", cam.get_exposure().unwrap());
                    println!("resolution: {:?}", cam.get_resolution().unwrap());
                }
                Err(e) => println!("failed to open camera, error: {}", e),
            }
        }
        Err(e) => println!("Failed, error code: {}", e),
    }
    //try streaming from our C11440_22CUSource interface
    viewer::Viewer::run(Settings::default()).expect("couldn't run app");
}
