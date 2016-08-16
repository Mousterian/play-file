extern crate libc;
extern crate coreaudio_sys;
pub use coreaudio_sys::core_audio;
mod error;
use error::Error;
use std::mem;
use std::ptr;

#[cfg(test)]
mod tests {

    use super::core_audio;

    #[test]
    fn it_works() {
        let file = String::from("/Users/paulsandison/paul/dev/rust_projects/coreaudio-rs/test.wav");
        println!("in it_works");
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
        println!("\n\nEverything is ok.");
        let audio_file_id = try!( super::open_audio_file(&file) );
        let data_format = try!( super::get_data_format(audio_file_id) );
        let graph = try!( super::new_au_graph() );

        let default_output_node = try!(super::graph_add_node(graph, core_audio::kAudioUnitType_Output,
                                                         core_audio::kAudioUnitSubType_DefaultOutput,
                                                         core_audio::kAudioUnitManufacturer_Apple));

        let file_node = try!(super::graph_add_node(graph, core_audio::kAudioUnitType_Generator,
                                                    core_audio::kAudioUnitSubType_AudioFilePlayer,
                                                    core_audio::kAudioUnitManufacturer_Apple));

        try!(super::graph_open(graph));

        let audio_unit = try!(super::graph_node_info(graph,file_node));

        try!(super::set_number_of_channels(audio_unit, core_audio::kAudioUnitScope_Output, 0, data_format.mChannelsPerFrame));

        try!(super::set_sample_rate(audio_unit, core_audio::kAudioUnitScope_Output, 0, data_format.mSampleRate));

        try!(super::graph_connect_node_input(graph, file_node, 0, default_output_node, 0));

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
pub fn graph_add_node(graph : core_audio::AUGraph, component_type : u32, component_sub_type : u32,
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

/// wraps AUGraphOpen
pub fn graph_open(graph : core_audio::AUGraph) -> Result<(), Error> {
    unsafe {
        try_os_status!(core_audio::AUGraphOpen(&mut *graph as core_audio::AUGraph));
        Ok(())
    }
}

/// wraps AUGraphNodeInfo
pub fn graph_node_info(graph : core_audio::AUGraph, node : core_audio::AUNode) -> Result<core_audio::AudioUnit, Error> {
    unsafe {
        let description: *mut core_audio::AudioComponentDescription = ptr::null_mut();
        let mut audio_unit : core_audio::AudioUnit = mem::uninitialized();
        
        
        match Error::from_os_status(core_audio::AUGraphNodeInfo(graph, node, description, &mut audio_unit)) {
            Ok(()) => {
                Ok(audio_unit)
            },
            Err(e) => Err(e)
        }
    }
}

pub fn graph_connect_node_input(graph : core_audio::AUGraph, source_node : core_audio::AUNode,
                                source_output : u32, dest_node : core_audio::AUNode,
                                dest_output : u32) -> Result<(),Error> {
    unsafe {
        try_os_status!(core_audio::AUGraphConnectNodeInput (graph, source_node, source_output, dest_node, dest_output));
        Ok(())
    }
}


pub fn set_number_of_channels ( audio_unit : core_audio::AudioUnit,
                                scope : core_audio::AudioUnitScope,
                                element : core_audio::AudioUnitElement,
                                number_of_channels: u32 ) -> Result<(), Error> {
    let mut description = try!(get_format(audio_unit, scope, element));
    change_number_channels(&mut description, number_of_channels);
    set_format(audio_unit, scope, element, &mut description)
}

pub fn set_sample_rate (audio_unit : core_audio::AudioUnit,
                        scope : core_audio::AudioUnitScope,
                        element : core_audio::AudioUnitElement,
                        sample_rate : f64) -> Result<(), Error> {

    let mut description = try!(get_format(audio_unit, scope, element));
    description.mSampleRate = sample_rate;
    set_format(audio_unit, scope, element, &mut description)
}

pub fn set_property (audio_unit : core_audio::AudioUnit,
                     property_id : core_audio::AudioUnitPropertyID,
                     scope : core_audio::AudioUnitScope,
                     element : core_audio::AudioUnitElement,
                     data : *const libc::c_void,
                     data_size : u32) -> Result<(),Error> {
    unsafe {
        try_os_status!(core_audio::AudioUnitSetProperty( audio_unit,
                                                         property_id,
                                                         scope,
                                                         element,
                                                         data,
                                                         data_size));
        Ok(())
    }
}

pub fn set_scheduled_file_ids (audio_unit : core_audio::AudioUnit,
                               scope : core_audio::AudioUnitScope,
                               element : core_audio::AudioUnitElement,
                               data : *const libc::c_void) -> Result<(),Error> {

    let property_size : u32 = mem::size_of::<core_audio::AudioStreamBasicDescription>() as u32;
    try!(set_property(audio_unit, core_audio::kAudioUnitProperty_ScheduledFileIDs,
                             scope, element, data, property_size));
    Ok(())
}

pub fn set_format(audio_unit : core_audio::AudioUnit,
                  scope : core_audio::AudioUnitScope,
                  element : core_audio::AudioUnitElement,
                  description : &core_audio::Struct_AudioStreamBasicDescription) -> Result<(),Error> {
    unsafe {
        let property_size : u32 = mem::size_of::<core_audio::AudioStreamBasicDescription>() as u32;
        try_os_status!(core_audio::AudioUnitSetProperty( audio_unit,
                                                         core_audio::kAudioUnitProperty_StreamFormat,
                                                         scope,
                                                         element,
                                                         description as *const _ as *const libc::c_void,
                                                         property_size));
        Ok(())
    }
}

pub fn get_format(  audio_unit : core_audio::AudioUnit,
                    scope : core_audio::AudioUnitScope,
                    element : core_audio::AudioUnitElement) -> Result<core_audio::AudioStreamBasicDescription, Error> {
    unsafe {
        let mut property_size : u32 = mem::size_of::<core_audio::AudioStreamBasicDescription>() as u32;
        let mut description : core_audio::Struct_AudioStreamBasicDescription = Default::default();
        try_os_status!(core_audio::AudioUnitGetProperty( audio_unit,
                                                         core_audio::kAudioUnitProperty_StreamFormat,
                                                         scope,
                                                         element,
                                                         &mut description as *mut _ as *mut libc::c_void,
                                                         &mut property_size));
        Ok(description)
    }
}

pub fn is_interleaved(description : &core_audio::AudioStreamBasicDescription) -> bool {
    let format_flags : i32 = description.mFormatFlags as i32;
    return !is_pcm(description) || (format_flags & core_audio::kAudioFormatFlagIsNonInterleaved == 0);
}

pub fn is_pcm(description : &core_audio::AudioStreamBasicDescription) -> bool {
    return description.mFormatID == core_audio::kAudioFormatLinearPCM;
}

pub fn change_number_channels(mut description: &mut core_audio::AudioStreamBasicDescription,
                              number_channels : u32) {
    let interleaved = is_interleaved(description);
    let mut word_size = sample_word_size(description);
    if word_size == 0 {
        word_size = (description.mBitsPerChannel + 7) /8;
    }

    description.mChannelsPerFrame = number_channels;
    description.mFramesPerPacket = 1;
    if interleaved {
        description.mBytesPerFrame = number_channels * word_size;
        description.mBytesPerPacket = description.mBytesPerFrame;
        // TO DO: this stinks, must be a better way - macro?
        let mut temp_format_flags = description.mFormatFlags as i32;
        temp_format_flags &= !core_audio::kAudioFormatFlagIsNonInterleaved;
        description.mFormatFlags = temp_format_flags as u32;
    }
    else {
        description.mBytesPerFrame = word_size;
        description.mBytesPerPacket = description.mBytesPerFrame;
        let mut temp_format_flags = description.mFormatFlags as i32;
        temp_format_flags |= core_audio::kAudioFormatFlagIsNonInterleaved;
        description.mFormatFlags = temp_format_flags as u32;
    }
}

pub fn sample_word_size(description : &core_audio::AudioStreamBasicDescription) -> u32 {
    let channels = number_interleaved_channels(description);
    if description.mBytesPerFrame > 0 && channels > 0 {
        description.mBytesPerFrame / channels
    }
    else {
        0
    }
}

pub fn number_interleaved_channels(description : &core_audio::AudioStreamBasicDescription) -> u32 {
    if is_interleaved(description) {
        description.mChannelsPerFrame
    }
    else {
        1
    }
}
