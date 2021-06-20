pub extern crate coreaudio_sys as sys;
use colored::*;
use std::mem::size_of;
use std::ptr;
use std::mem;
use std::os::raw::c_void;
use sys::OSStatus;
use quicli::prelude::*;
use structopt::StructOpt;
use plotters::prelude::*;

mod console;

static mut audiodata:Vec<f32> = vec![];

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
    unsafe {
        assert!(_in_number_frames == audiodata.len() as u32);
        let ptr = (*_io_data).mBuffers.as_ptr() as *mut sys::AudioBuffer;
        let len = (*_io_data).mNumberBuffers as usize;
        let buffers = std::slice::from_raw_parts_mut(ptr, len);
        for i in 0..2{
            buffers[i as usize].mData = audiodata.as_ptr() as *mut c_void;
        }
    }
    return 0 as sys::OSStatus
}

//------------------------------------------------
//------------CLI STUFF---------------------------
//------------------------------------------------

#[derive(Debug, StructOpt)]
struct Cli {
  #[structopt(long="noplot")]
  noplot: bool
}

//------------------------------------------------
//------------WAVE DATA---------------------------
//------------------------------------------------
// Note: 440 is A
pub fn make_sine() -> Vec<f64> {
  (0..512)
      .map(|x| (2f64 * std::f64::consts::PI * x as f64) / 512.0)
      .map(|x| x.sin())
      .collect()
}

pub fn add_sine(signal : &mut Vec<f64>, freq: i16, amp: f64, phase: f64) {
  for i in 0..signal.len() {
    let mut new_signal;
    new_signal = std::f64::consts::PI * 2.0 * (i as f64 * freq as f64);
    new_signal = new_signal / 512.0;
    new_signal += phase;
    new_signal = new_signal.sin() * amp; 
    signal[i] += new_signal;
  }
}

pub fn make_square() -> Vec<f64> {
  let wave = &mut make_sine();
  let updates = 3..50;
  for i in updates.step_by(2) {
    add_sine(
        wave,
        i,
        1.0 / i as f64,
        0f64
    );
  }
  wave.to_vec()
}

fn draw(data: Vec<f64>) {
  let drawing_area = console::TextDrawingBackend(vec![console::PixelState::Empty; 5000]) 
    .into_drawing_area();

  let _x = console::draw_chart(drawing_area, data);
  return;
}

fn main() {
    let args = Cli::from_args();

    if !args.noplot {
        println!("In plot mode");
        let data = make_square();
        draw(data);
        println!("I drew");
    } else {
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

        unsafe {
            audiodata = make_square().iter().map(|x| *x as f32).collect();
            let component = sys::AudioComponentFindNext(ptr::null_mut(), &desc as *const _); // what does *const _ mean?)
            if component.is_null() {
                println!("{}", apple_said_no(&"Couldn't find a component"));
            } else {
                println!("{}", apple_said_yes(&"Could find a component")); 
            }
            let mut instance_uninit = mem::MaybeUninit::<sys::AudioUnit>::uninit();
            let status: OSStatus = sys::AudioComponentInstanceNew(component,
                instance_uninit.as_mut_ptr() as *mut sys::AudioUnit);
            if status == 0 as OSStatus {
                println!("{}", apple_said_yes(&"We made an audio instance"));
            } else {
                println!("{}", apple_said_no(&String::from(format!("{}, {}", "We failed to make an audio instance", status))));
            }

            let instance = instance_uninit.assume_init();

            let initalize: OSStatus = sys::AudioUnitInitialize(instance);
            if initalize == 0 as OSStatus {
                println!("{}", apple_said_yes(&"We initialized an audio instance"));
            } else {
                println!("{}", apple_said_no(&String::from(format!("{}, {}", "We failed to make an audio instance", initalize))));
            }

            // Wire it up to something?
            // describe the stream 
            let from_existing = false;
            #[allow(dead_code)]
            if from_existing
            {
                let id = sys::kAudioUnitProperty_SampleRate;
                let mut data_uninit = ::std::mem::MaybeUninit::<f32>::uninit();
                let mut size = ::std::mem::size_of::<f32>() as u32;
                let data_ptr = data_uninit.as_mut_ptr() as *mut _ as *mut c_void;
                let size_ptr = &mut size as *mut _;

                let fetchSampleRate = sys::AudioUnitGetProperty(instance, id, 1,  0, data_ptr, size_ptr);
                if status == 0 as OSStatus {
                    println!("{}", apple_said_yes("We got a thing"));
                }
            }

            // set the thing up like this https://stackoverflow.com/a/36970515
            let stream_desc = sys::AudioStreamBasicDescription {
                mReserved: 0,
                mBytesPerFrame: size_of::<f32>() as u32,
                mBytesPerPacket: size_of::<f32>() as u32,
                mBitsPerChannel: size_of::<f32>() as u32 * 8,
                mFormatID: sys::kAudioFormatLinearPCM,
                mFormatFlags: sys::kAudioFormatFlagIsSignedInteger | sys::kAudioFormatFlagIsPacked,
                mChannelsPerFrame: 1, // making it mono
                mFramesPerPacket: 1,
                mSampleRate: 48000 as f64 // 48khz,
            };

            let mut render_callback = sys::AURenderCallbackStruct {
                inputProc: Some(my_input_wrapper),
                inputProcRefCon: my_input_wrapper as *mut c_void
            };

            let render_callback_ref = &mut render_callback as *mut _ as *mut c_void;
            sys::AudioUnitSetProperty(
                instance, 
                sys::kAudioUnitProperty_SetRenderCallback, 
                1, 
                0, 
                render_callback_ref,
                ::std::mem::size_of::< sys::AURenderCallbackStruct >() as u32
            );

            // Profit??
            // Well, we set a render callback that will do the work, so not a whole lot of profit.
            let try_start = sys::AudioOutputUnitStart(instance);
            if try_start == 0 as OSStatus {
                println!("{}: {}", apple_said_yes(&"Starting!"), try_start);
                std::thread::sleep(std::time::Duration::from_millis(3000));
            } else {
                panic!("{}", apple_said_no(&"Shit"));
            }

            // Clean up?
            let uninitalize: OSStatus = sys::AudioUnitUninitialize(instance);
            if uninitalize == 0 as OSStatus {
                println!("{}", apple_said_yes(&"We unmade a audio instance"));
            } else {
                println!("{}", apple_said_no(&String::from(format!("{}, {}", "We failed to unmake an audio instance", uninitalize))));
            }
        }
    }
}
