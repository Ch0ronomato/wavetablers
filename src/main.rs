pub extern crate coreaudio_sys as sys;
use colored::*;
use std::mem::size_of;
use std::ptr;
use std::mem;
use std::os::raw::c_void;
use sys::OSStatus;
use structopt::StructOpt;
use plotters::prelude::*;

mod console;

static mut FINALIZED_DATA:Vec<f64> = vec![];
static mut SHOULD_MUTE:f32 = 0.0f32;
static mut AUDIODATA:Vec<f32> = vec![];
static mut AUDIODATAINDEX: usize = 0;

//------------------------------------------------
//------------APPLE STUFF-------------------------
//------------------------------------------------
pub fn apple_said(what: &str, more_info: &str) -> String {
    let s = match what {
        "yes" => format!("{}: {}", "Apple said yes", more_info).green().bold().to_string(),
        "no" =>  format!("{}: {}", "Apple said no", more_info).red().bold().to_string(),
        _ => String::from("")
    };
    s.to_string()
}

pub fn apple_said_no(more_info: &str) -> String {
    return apple_said("no", more_info);
}

pub fn apple_said_yes(more_info: &str) -> String {
    return apple_said("yes", more_info);
}

extern "C" fn my_input_wrapper(_in_ref_con: *mut c_void,
    _flags: *mut sys::AudioUnitRenderActionFlags,
    _time: *const sys::AudioTimeStamp,
    _in_bus_number: sys::UInt32,
    _in_number_frames: sys::UInt32,
    _io_data: *mut sys::AudioBufferList) -> sys::OSStatus {
    let d = unsafe { &mut *_io_data };
    let channels_ptr = d.mBuffers.as_ptr() as *mut sys::AudioBuffer;
    let channels_len = d.mNumberBuffers as usize;
    let buffers = unsafe { std::slice::from_raw_parts_mut(channels_ptr, channels_len) };
    for i in 0..channels_len {
        let buff_size = _in_number_frames as usize * channels_len;
        let ptr = buffers[i as usize].mData as *mut f32;
        let data = unsafe { std::slice::from_raw_parts_mut(ptr, buff_size) };
        for j in 0..buff_size {
            let index = unsafe {
                if AUDIODATAINDEX + j >= AUDIODATA.len() {
                    AUDIODATAINDEX = 0;
                    AUDIODATAINDEX
                } else {
                    AUDIODATAINDEX + j
                }
            };
            unsafe {
                assert!(index < AUDIODATA.len());
                data[j] = AUDIODATA[index];
                FINALIZED_DATA.push(data[j] as f64);
                data[j] *= SHOULD_MUTE; // muting for now
                AUDIODATAINDEX = AUDIODATAINDEX + 1;
            }
        }
    }
    return 0 as sys::OSStatus
}

//------------------------------------------------
//------------CLI STUFF---------------------------
//------------------------------------------------

#[derive(Debug, StructOpt)]
struct Cli {
  #[structopt(long="nomute")]
  nomute: bool
}

//------------------------------------------------
//------------WAVE DATA---------------------------
//----------------------square--------------------------
// Note: 440 is A
const FREQF: f64 = 880.0f64;
const SAMPLE_RATE: f64 = 880.0f64;
pub fn make_sine() -> Vec<f64> {
  let cycles_per_sample = FREQF / SAMPLE_RATE;
  let angle_delta = cycles_per_sample * std::f64::consts::PI * 2.0f64;
  (0..SAMPLE_RATE as usize)
      .map(|x| (angle_delta * x as f64) / SAMPLE_RATE)
      .map(|x| x.sin())
      .collect()
}

pub fn add_sine(signal : &mut Vec<f64>, freq: f64, amp: f64, phase: f64) {
  let twopi = std::f64::consts::PI * 2.0f64;

  // audiosignal[i]+= amp * sin((TWO_PI * (i*freq) / 512) + phase);
  for i in 0..SAMPLE_RATE as usize {
    let mut new_signal;
    new_signal = freq * i as f64;
    new_signal = new_signal / SAMPLE_RATE;
    new_signal = new_signal * twopi;
    new_signal += phase;
    new_signal = new_signal.sin() * amp; 
    signal[i] += new_signal;
  }
}

pub fn make_square() -> Vec<f64> {
  let wave = &mut make_sine();
  let updates = 3..(20_000.0f64 - FREQF) as i32;
  for i in updates.step_by(2) {
      let i_f = i as f64;
      add_sine(
          wave,
          FREQF + i_f,
          1.0 / i_f,
          0f64
      );
  }
  wave.to_vec()
}

fn draw_console(data: &Vec<f64>) {
  let drawing_area = console::TextDrawingBackend(vec![console::PixelState::Empty; 5000]) 
    .into_drawing_area();

  let _x = console::draw_chart(drawing_area, data.to_vec(), FREQF);
  return;
}

fn main() {
    let args = Cli::from_args();

    if args.nomute {
        unsafe {
            SHOULD_MUTE = 1.00f32;
        }
    }
    unsafe { AUDIODATA = make_square().iter().map(|x| *x as f32).collect(); }
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
        mBytesPerFrame: size_of::<f32>() as u32,
        mBytesPerPacket: size_of::<f32>() as u32,
        mBitsPerChannel: size_of::<f32>() as u32 * 8,
        mFormatID: sys::kAudioFormatLinearPCM,
        mFormatFlags: sys::kAudioFormatFlagIsSignedInteger | sys::kAudioFormatFlagIsPacked,
        mChannelsPerFrame: 1, // making it mono
        mFramesPerPacket: 1,
        mSampleRate: SAMPLE_RATE
    };


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
        draw_console(&FINALIZED_DATA);
    }
}
