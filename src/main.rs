pub extern crate coreaudio_sys as sys;

use colored::*;
use std::ptr;
use std::mem;
// use std::os::raw::{c_uint, c_void};
use sys::OSStatus;


pub fn appleSaid(what: &str, moreInfo: &str) -> String {
    let s = match what {
        "yes" => format!("{}: {}", "Apple said yes", moreInfo).green().bold().to_string(),
        "no" =>  format!("{}: {}", "Apple said no", moreInfo).red().bold().to_string(),
        _ => String::from("")
    };
    s.to_string()
}

pub fn appleSaidNo(moreInfo: &str) -> String {
    return appleSaid("no", moreInfo);
}

pub fn appleSaidYes(moreInfo: &str) -> String {
    return appleSaid("yes", moreInfo);
}

fn main() {
    // initialize the AU
    const MANUFACTURER_IDENTIFIER: u32 = sys::kAudioUnitManufacturer_Apple; // Apple wants everything signed
    const AUDIO_TYPE: u32 = 1635086197; // Indicates our AU will make sound
    const AUDIO_SUBTYPE: u32 = 1734700658; // Indiciates it uses the default sound device
    
    let desc = sys::AudioComponentDescription {
        componentType: AUDIO_TYPE,
        componentSubType: AUDIO_SUBTYPE,
        componentManufacturer: MANUFACTURER_IDENTIFIER,
        componentFlags: 0,
        componentFlagsMask: 0
    };

    unsafe {
        let component = sys::AudioComponentFindNext(ptr::null_mut(), &desc as *const _); // what does *const _ mean?)
        if component.is_null() {
            println!("{}", appleSaidNo(&"Couldn't find a component"));
        } else {
            println!("{}", appleSaidYes(&"Could find a component")); 
        }
        let mut instance_uninit = mem::MaybeUninit::<sys::AudioUnit>::uninit();
        let status: OSStatus = sys::AudioComponentInstanceNew(component,
            instance_uninit.as_mut_ptr() as *mut sys::AudioUnit);
        if status == 0 as OSStatus {
            println!("{}", appleSaidYes(&"We made an audio instance"));
        } else {
            println!("{}", appleSaidNo(&String::from(format!("{}, {}", "We failed to make an audio instance", status))));
        }

        let instance = instance_uninit.assume_init();

        let initalize: OSStatus = sys::AudioUnitInitialize(instance);
        if initalize == 0 as OSStatus {
            println!("{}", appleSaidYes(&"We initialized an audio instance"));
        } else {
            println!("{}", appleSaidNo(&String::from(format!("{}, {}", "We failed to make an audio instance", initalize))));
        }

        // Wire it up to something?

        // Profit??

        // Clean up?
        let uninitalize: OSStatus = sys::AudioUnitUninitialize(instance);
        if uninitalize == 0 as OSStatus {
            println!("{}", appleSaidYes(&"We unmade a audio instance"));
        } else {
            println!("{}", appleSaidNo(&String::from(format!("{}, {}", "We failed to unmake an audio instance", uninitalize))));
        }
        // Ok(()); Do I not need this?
    }


    // Ok(());
}
