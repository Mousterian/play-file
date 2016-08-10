extern crate libc;
extern crate coreaudio_sys;
pub use coreaudio_sys::core_audio;
mod error;
use error::Error;
use std::mem;

#[cfg(test)]
mod tests {

    use super::core_audio;

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
        let graph = try!( super::new_au_graph() );

        let _default_output_node = try!(super::add_node(graph, core_audio::kAudioUnitType_Output,
                                                         core_audio::kAudioUnitSubType_DefaultOutput,
                                                         core_audio::kAudioUnitManufacturer_Apple));

        let _file_node = try!(super::add_node(graph, core_audio::kAudioUnitType_Generator,
                                                    core_audio::kAudioUnitSubType_AudioFilePlayer,
                                                    core_audio::kAudioUnitManufacturer_Apple));

        try!(super::graph_open(graph));

        // TO DO: wrap this in a trait and implement drop for automatic release
        super::drop_au_graph(graph);
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

pub fn new_au_graph() -> Result<core_audio::AUGraph, Error> {
    unsafe {
        let mut graph = mem::uninitialized();

        match Error::from_os_status(core_audio::NewAUGraph (&mut graph as *mut core_audio::AUGraph)) {
            Ok(()) => {
                Ok( graph )
            }
            Err(err) => {
                // TO DO: wrap this in a RAII guard object and move this out of here?
                drop_au_graph(graph);
                Err(err)
            }
        }
    }
}

pub fn drop_au_graph(instance : core_audio::AUGraph) {
    unsafe {
        use error;
        use std::error::Error;

        if let Err(err) = error::Error::from_os_status(core_audio::AUGraphStop(instance)) {
            panic!("{:?}", err.description());
        }
        if let Err(err) = error::Error::from_os_status(core_audio::AUGraphUninitialize(instance)) {
            panic!("{:?}", err.description());
        }
        if let Err(err) = error::Error::from_os_status(core_audio::AUGraphClose(instance)) {
            panic!("{:?}", err.description());
        }
    }
}

/// wraps AUGraphAddNode
pub fn add_node(graph : core_audio::AUGraph, component_type : u32, component_sub_type : u32,
                manufacturer: u32) -> Result<core_audio::AUNode,Error> {
    unsafe {
        let description = core_audio::AudioComponentDescription { 	componentType: component_type,
                                                            componentSubType: component_sub_type,
                                                            componentManufacturer: manufacturer,
                                                            /* TO DO: figure out does anybody actually use these? */
                                                            componentFlags: 0,
                                                            componentFlagsMask: 0 };
        let mut node: core_audio::AUNode = mem::uninitialized();
        match Error::from_os_status(core_audio::AUGraphAddNode(graph,
                                    &description as *const core_audio::AudioComponentDescription,
                                    &mut node as *mut core_audio::AUNode)) {
            Ok(()) => Ok(node),
            Err(e) => Err(e)
        }
    }
}

pub fn graph_open(graph : core_audio::AUGraph) -> Result<(), Error> {
    unsafe {
        try_os_status!(core_audio::AUGraphOpen(&mut *graph as core_audio::AUGraph));
        Ok(())
    }
}