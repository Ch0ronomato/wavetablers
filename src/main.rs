pub extern crate coreaudio_sys as sys;
use std::mem::size_of;
use structopt::StructOpt;
use std::ptr;
use std::mem;
use std::os::raw::c_void;
use std::fs;
use sys::OSStatus;
use util::apple_said_yes;
use util::apple_said_no;

mod console;
mod sounds;
mod util;

static mut FINALIZED_DATA:Vec<f64> = vec![];
static mut SHOULD_MUTE:f32 = 0.0f32;
static mut AUDIODATA:Vec<f32> = vec![];
static mut AUDIODATAINDEX: usize = 0;

// Note: 440 is A
const FREQF: f64 = 440.0f64;
const SAMPLE_RATE: f64 = 48_000.0f64;

extern "C" fn my_input_wrapper(_in_ref_con: *mut c_void,
    _flags: *mut sys::AudioUnitRenderActionFlags,
    _time: *const sys::AudioTimeStamp,
    _in_bus_number: sys::UInt32,
    _in_number_frames: sys::UInt32,
    _io_data: *mut sys::AudioBufferList) -> sys::OSStatus {
    let d = unsafe { &mut *_io_data };
    let channels_ptr = d.mBuffers.as_ptr() as *mut sys::AudioBuffer;
    let channels_len = d.mNumberBuffers as usize;
    assert!(channels_len == 2, "Not in stereo mode yet");
    let buffers = unsafe { std::slice::from_raw_parts_mut(channels_ptr, channels_len) };
    let buff_size = _in_number_frames as usize * channels_len;
    let channel_data: &mut Vec<_> = &mut buffers
        .iter()
        .map(|buffer| buffer.mData as *mut f32)
        .flat_map(|data_for_channel| unsafe { std::slice::from_raw_parts_mut(data_for_channel, buff_size) })
        .collect::<Vec<_>>();
    for i in 0..buff_size {
        let point: f32 = unsafe { AUDIODATA[(AUDIODATAINDEX + i) % AUDIODATA.len()] } * 0.5f32;
        for channel in 0..channels_len {
            *channel_data[(buff_size * channel) + i] = point;
        }
        unsafe {
                FINALIZED_DATA.push(point as f64);
        }
    }
    unsafe { AUDIODATAINDEX = (AUDIODATAINDEX + buff_size) % AUDIODATA.len() };
    return 0 as sys::OSStatus
}


fn main() {
    let args = util::Cli::from_args();

    if args.nomute {
        unsafe {
            SHOULD_MUTE = 1.00f32;
        }
    }
    unsafe { AUDIODATA = sounds::make_sine().iter().map(|x| *x as f32).collect(); }
    // initialize the AU
    const MANUFACTURER_IDENTIFIER: u32 = sys::kAudioUnitManufacturer_Apple; // Apple wants everything signed
    const AUDIO_TYPE: u32 = sys::kAudioUnitType_Output; // Indicates our AU will make sound
    const AUDIO_SUBTYPE: u32 = sys::kAudioUnitSubType_DefaultOutput; // Indiciates it uses the default sound device
    let desc = sys::AudioComponentDescription {
        componentType: AUDIO_TYPE,
        componentSubType: AUDIO_SUBTYPE,
        componentManufacturer: MANUFACTURER_IDENTIFIER,
        componentFlags: 0,
        componentFlagsMask: 0
    };
    // set the thing up like this https://stackoverflow.com/a/36970515
    let mut stream_desc = sys::AudioStreamBasicDescription {
        mReserved: 0,
        mBytesPerFrame: size_of::<f32>() as u32 * 2,
        mBytesPerPacket: size_of::<f32>() as u32 * 2,
        mBitsPerChannel: size_of::<f32>() as u32 * 8,
        mFormatID: sys::kAudioFormatLinearPCM,
        mFormatFlags: sys::kAudioFormatFlagIsFloat | sys::kAudioFormatFlagIsPacked,
        mChannelsPerFrame: 2,
        mFramesPerPacket: 1,
        mSampleRate: SAMPLE_RATE
    };
    println!("{:#?} {:#?}", desc, stream_desc);


    let component = unsafe { sys::AudioComponentFindNext(ptr::null_mut(), &desc as *const _) };
    if component.is_null() {
        println!("{}", apple_said_no(&"Couldn't find a component"));
    } else {
        println!("{}", apple_said_yes(&"Could find a component"));
    }
    let mut instance_uninit = mem::MaybeUninit::<sys::AudioUnit>::uninit();
    let status: OSStatus = unsafe {
        sys::AudioComponentInstanceNew(component,instance_uninit.as_mut_ptr() as *mut sys::AudioUnit)
    };
    if status == 0 as OSStatus {
        println!("{}", apple_said_yes(&"We made an audio instance"));
    } else {
        println!("{}", apple_said_no(&String::from(format!("{}, {}", "We failed to make an audio instance", status))));
    }

    let instance = unsafe { instance_uninit.assume_init() };

    let initalize: OSStatus = unsafe { sys::AudioUnitInitialize(instance) };
    if initalize == 0 as OSStatus {
        println!("{}", apple_said_yes(&"We initialized an audio instance"));
    } else {
        println!("{}", apple_said_no(&String::from(format!("{}, {}", "We failed to make an audio instance", initalize))));
    }
    let mut render_callback = sys::AURenderCallbackStruct {
        inputProc: Some(my_input_wrapper),
        inputProcRefCon: my_input_wrapper as *mut c_void
    };

    let render_callback_ref = &mut render_callback as *mut _ as *mut c_void;
    let description_ref = &mut stream_desc as *mut _ as *mut c_void;
    let try_start = unsafe {
        sys::AudioUnitSetProperty (
            instance,
            sys::kAudioUnitProperty_StreamFormat,
            1,
            0,
            description_ref,
            ::std::mem::size_of::< sys::AudioStreamBasicDescription >() as u32
        );

        sys::AudioUnitSetProperty(
            instance,
            sys::kAudioUnitProperty_SetRenderCallback,
            1,
            0,
            render_callback_ref,
            ::std::mem::size_of::< sys::AURenderCallbackStruct >() as u32
        );
        sys::AudioOutputUnitStart(instance)
    };

    // Profit??
    // Well, we set a render callback that will do the work, so not a whole lot of profit.
    if try_start == 0 as OSStatus {
        println!("{}: {:?}", apple_said_yes(&"Starting!"), try_start);
        std::thread::sleep(std::time::Duration::from_millis(3000));
    } else {
        panic!("{}", apple_said_no(&"Shit"));
    }

    // Clean up?
    let uninitalize: OSStatus = unsafe { sys::AudioUnitUninitialize(instance) };
    if uninitalize == 0 as OSStatus {
        println!("{}", apple_said_yes(&"We unmade a audio instance"));
    } else {
        println!("{}", apple_said_no(&String::from(format!("{}, {}", "We failed to unmake an audio instance", uninitalize))));
    }

    // Print.
    unsafe {
        console::draw_console(&FINALIZED_DATA);
    }
    let outd: String = unsafe { 
        (0..AUDIODATAINDEX) 
            .map(|x| FINALIZED_DATA[x].to_string())
            .fold(String::new(), |acc, x| {
               acc + "\n" + &x 
            })
    };
    fs::write("mywave.csv", outd).expect("wanted to dump this");
}
