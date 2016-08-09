extern crate libc;
extern crate coreaudio_sys;
pub use coreaudio_sys::core_audio;
mod error;
use error::Error;
use std::mem;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        let file = String::from("/Users/paulsandison/paul/dev/rust_projects/coreaudio-rs/test.wav");
        let result = play_file(&file);

        match result {
            Ok(_) => {
                println!("\n\nEverything is ok.");
            },
            Err(err) => {
                panic!("Could not play file, error: {:?}", err);
            }
        }
    }

    fn play_file(file: &String) -> Result<(),super::error::Error> {
        let audio_file_id = try!( super::open_audio_file(&file) );
        try!( super::get_data_format(audio_file_id) );
        Ok(())
    }
}

macro_rules! try_os_status {
($expr:expr) => (try!(Error::from_os_status($expr)))
}

pub fn open_audio_file(path: &String) -> Result<core_audio::AudioFileID, Error> {
    unsafe {
        let url_ref = try!( match core_audio::CFURLCreateFromFileSystemRepresentation(core_audio::kCFAllocatorDefault,
                                                                              path.as_ptr(),
                                                                              path.len() as i64,
                                                                              0 as core_audio::Boolean) {
            url_ref if url_ref.is_null()    => Err(Error::Unspecified),
            url_ref                         => Ok(url_ref),
        } );

        let mut audio_file_id: core_audio::AudioFileID = mem::uninitialized();
        try_os_status!(core_audio::AudioFileOpenURL(url_ref,
                    core_audio::kAudioFileReadPermission as i8,
                    0,
                    &mut audio_file_id as *mut core_audio::AudioFileID));

        core_audio::CFRelease(url_ref as core_audio::CFTypeRef);
        Ok(audio_file_id)
    }
}

pub fn get_data_format(audio_file_id: core_audio::AudioFileID) -> Result<core_audio::AudioStreamBasicDescription, Error> {
    unsafe {
        // get the number of channels of the file
        let mut file_format : core_audio::AudioStreamBasicDescription = mem::uninitialized();
        let mut property_size = mem::size_of::<core_audio::AudioStreamBasicDescription>() as u32;
        try_os_status!(core_audio::AudioFileGetProperty(audio_file_id,
                                                core_audio::kAudioFilePropertyDataFormat,
                                                &mut property_size as *mut core_audio::UInt32,
                                                &mut file_format as *mut _ as *mut libc::c_void));
        Ok(file_format)
    }
}