fn main() {
    //let's try initializing the API and see what happens
    //let mut guid = bindings::DCAM_GUID::new();
    let api_result = free_willy::DcamAPI::connect();
    match api_result {
        Ok(api) => println!("Connected, detecting {} camera", api.ncam()),
        Err(e) => println!("Failed, error code: {}", e),
    }
}
