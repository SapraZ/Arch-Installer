#![allow(dead_code, unused)]

use ansi_term::Colour::Green;
use block_utils;
use bytesize::{self, ByteSize};
use chrono_tz;
pub use console;
use console::{Color, Term};
use dialoguer;
use std::fmt::{self, Error};
use std::process::Command;
use std::{io, ops::BitAndAssign};
use derivative::Derivative;
use online;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(EnumIter, Debug, PartialEq)]
enum Kernel {
    Linux,
    LinuxLTS,
    LinuxXanmod,
}
impl std::fmt::Display for Kernel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Kernel::Linux => write!(f, "linux"),
            Kernel::LinuxLTS => write!(f, "linux-lts"),
            Kernel::LinuxXanmod => write!(f, "linux-xanmod"),
        }
    }
}
enum FileSystem {
    Ext4,
    Btrfs,
}
enum PartitionTypes {
    Efi,
    Swap,
    Root,
    Home,
}
struct Partition {
    size: usize,
    partition_type: PartitionTypes,
    file_system: FileSystem,
}

enum DriveType {
    Nvme,
    Ssd,
    Hdd,
    Loopback,
    Lvm,
    MdRaid,
    Ram,
    Virtual,
    Unknown,
}

impl fmt::Display for DriveType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            DriveType::Nvme => write!(f, "nvme"),
            DriveType::Ssd => write!(f, "ssd"),
            DriveType::Virtual => write!(f, "virtual"),
            DriveType::Ram => write!(f, "ram"),
            DriveType::Hdd => write!(f, "hdd"),
            DriveType::Loopback => write!(f, "loopback"),
            DriveType::Lvm => write!(f, "lvm"),
            DriveType::MdRaid => write!(f, "mdraid"),
            _ => write!(f, "unknown"),
        }
    }
}

struct Drive {
    name: String,
    drive_type: DriveType,
    size: u64,
    partitions: Vec<Partition>,
}

struct Profile {
    drives: Vec<Drive>,
    kernel: Kernel,
    time_zone: String,
    keyboard_layout: String,
    host_name: String,
    user_name: String,
}

impl Default for Profile {
    fn default() -> Profile {
        Profile {
            kernel: Kernel::Linux,
            user_name: "user".to_string(),
            host_name: "ArchLinux".to_string(),
            keyboard_layout: "se-lat6".to_string(),
            time_zone: "UTC+0".to_string(),
            drives: Vec::new(),
        }
    }
}

fn convert_drive_type(old: block_utils::MediaType) -> DriveType {
    match old {
        block_utils::MediaType::NVME => return DriveType::Nvme,
        block_utils::MediaType::SolidState => return DriveType::Ssd,
        block_utils::MediaType::Rotational => return DriveType::Hdd,
        block_utils::MediaType::Ram => return DriveType::Ram,
        block_utils::MediaType::Loopback => return DriveType::Loopback,
        block_utils::MediaType::MdRaid => return DriveType::MdRaid,
        block_utils::MediaType::Virtual => return DriveType::Virtual,
        block_utils::MediaType::LVM => return DriveType::Lvm,
        block_utils::MediaType::Unknown => return DriveType::Unknown,
        _ => println!("Unknown disktype"),
    };
    return DriveType::Unknown;
}

fn ask_about_new_profile() -> std::io::Result<()> {
    println!("Welcome to ArchInstaller 0.1");
    if dialoguer::Confirm::new()
        .with_prompt("Do you want to create a new profile?")
        .interact()?
    {
        println!("Looks like you want to continue");
    } else {
        println!("nevermind then :(");
    }
    return Ok(());
}

fn print_profile(profile: &Profile) {
    println!("-=[ Profile ]=-");
    println!("- Kernel: {}", profile.kernel);
    println!("- Drives");
    for drive in &profile.drives {
        println!("  | {} {}", drive.name, ByteSize::b(drive.size));
    }
}

fn partition_drives() -> Vec<Drive> {
    let mut drives: Vec<Drive> = vec![];

    fn get_drives() -> Vec<Drive> {
        let mut found_drives: Vec<Drive> = vec![];
        let block_devices = block_utils::get_block_devices();
        let devices = block_devices.ok().unwrap();
        for i in devices {
            let device = block_utils::get_device_info(&i).unwrap();
            let new_device = Drive {
                name: device.name,
                size: device.capacity,
                partitions: vec![],
                drive_type: convert_drive_type(device.media_type),
            };
            found_drives.push(new_device);
        }
        return found_drives;
    }

    fn select_drive(drives: &Vec<Drive>) -> std::io::Result<()> {
        let mut drive_names: Vec<String> = vec![];
        for drive in drives {
            drive_names.push(drive.name.to_string());
        }
        let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .items(&drive_names)
            .default(0)
            .interact_on_opt(&console::Term::stderr())?;

        match selection {
            Some(index) => println!("User selected item : {}", drive_names[index]),
            None => println!("User did not select anything"),
        }

        Ok(())
    }

    fn format_drive() {}

    drives = get_drives();
    select_drive(&drives);
    return drives;
}

fn get_timezones() -> Vec<String> {
    use std::str::Lines;

    let bytes = std::include_bytes!("timezones.txt");
    print!("{}", String::from_utf8_lossy(bytes));
    let full_string: String = String::from_utf8_lossy(bytes).to_string();
    let time_zones: Lines = full_string.lines();

    let mut zones: Vec<String> = vec![];

    for line in time_zones {
        zones.push(line.to_string());
    }

    return zones;
}

#[cfg(not(debug_assertions))]
fn get_timezones2() -> Vec<String> {
    if let Ok(profile) = std::env::var("PROFILE") {
        println!("cargo:rustc-cfg=build={:?}", profile);
    }

    let mut time_zones: Vec<String> = vec![];

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "echo hello"])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("timedatectl list-timezones")
            .output()
            .expect("failed to execute process")
    };

    let hello = output.stdout;

    println!("{}", String::from_utf8_lossy(&hello));

    return time_zones;
}

fn select_timezone(mut prof: Profile) -> Profile {
    let time_zones: Vec<String> = get_timezones();

    let selected_time_zone = fuzzy_select_timezone(time_zones);

    match selected_time_zone {
        Ok(value) => {
            prof.time_zone = value;
            return  prof;
        },
        Err(error) => panic!("Problem selecting tz: {:?}", error),
    }
}

fn fuzzy_select_timezone(time_zones: Vec<String>) -> std::io::Result<String> {
    let selection = dialoguer::FuzzySelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&time_zones)
        .default(0)
        .interact_on_opt(&console::Term::stderr())?;
    return Ok(time_zones[selection.unwrap()].clone());
}

#[cfg(debug_assertions)]
fn get_keyboard_layout() -> Vec<String> {
    use std::str::Lines;

    let bytes = std::include_bytes!("layouts.txt");
    print!("{}", String::from_utf8_lossy(bytes));
    let full_string: String = String::from_utf8_lossy(bytes).to_string();
    let time_zones: Lines = full_string.lines();

    let mut zones: Vec<String> = vec![];

    for line in time_zones {
        zones.push(line.to_string());
    }

    return zones;
}

#[cfg(not(debug_assertions))]
fn get_keyboard_layout() -> Vec<String> {
    if let Ok(profile) = std::env::var("PROFILE") {
        println!("cargo:rustc-cfg=build={:?}", profile);
    }

    let mut time_zones: Vec<String> = vec![];

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "echo hello"])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("localectl list-keymaps")
            .output()
            .expect("failed to execute process")
    };

    let hello = output.stdout;

    println!("{}", String::from_utf8_lossy(&hello));

    return time_zones;
}

fn select_keyboard_layout() -> String {
    let time_zones: Vec<String> = get_keyboard_layout();

    let selected_time_zone = fuzzy_select_keyboard_layout(time_zones);

    match selected_time_zone {
        Ok(value) => return value,
        Err(error) => panic!("Problem selecting layout: {:?}", error),
    }
}

fn fuzzy_select_keyboard_layout(time_zones: Vec<String>) -> std::io::Result<String> {
    let selection = dialoguer::FuzzySelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&time_zones)
        .default(0)
        .interact_on_opt(&console::Term::stderr())?;
    return Ok(time_zones[selection.unwrap()].clone());
}

#[derive(Derivative)]
#[derivative(Debug, Default)]
struct MenuChoise {
    name: String,
    id: i32,
    #[derivative(Default(value = "false"))]
    valid: bool,
    #[derivative(Default(value = "false"))]
    required: bool
}

fn update_validity(profile:&Profile,mut choises: Vec<MenuChoise>) -> Vec<MenuChoise>{
    
    println!("Checking validity");
    

    for choise in &mut choises {
        println!("{}",choise.id);
        match choise.id {
            1 => {
                println!("Checking timezone");
                if get_timezones().contains(&profile.time_zone) {
                    choise.valid = true;
                }
            }
            2 => {
                choise.valid = get_keyboard_layout().contains(&profile.keyboard_layout)
            }
            4 => {
                choise.valid = check_network_status();
            }
            _ => (),
        };
    }

    
    return choises;
}

fn check_network_status() -> bool {
    return online::check(None).is_ok();
}

fn select_selection_index_from_strings(strings: Vec<String>) -> usize{
    let selection = dialoguer::FuzzySelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&strings)
        .default(0)
        .interact_on_opt(&console::Term::stderr());
    match selection {
        Ok(Some(index)) => return index.try_into().unwrap(),
        Ok(None) => panic!("Problem getting selection"),
        Err(error) => panic!("Problem getting selection: {:?}", error),
    };
    
}

fn get_kernel_strings() -> Vec<String> {
    let mut strings: Vec<String> = vec![];
    for kernels in Kernel::iter() {
        strings.push(kernels.to_string())
    }
    return strings;
}

fn options_panel(mut profile: Profile) {
    // Example

    let mut choises: Vec<MenuChoise> = vec![];
    choises.push(MenuChoise{name:"Select keyboard Layout".to_string(),id:2,..Default::default()});
    choises.push(MenuChoise{name:"Select TimeZone".to_string(),id:1,..Default::default()});
    choises.push(MenuChoise{name:"Partition Drives".to_string(),id:3,..Default::default()});
    choises.push(MenuChoise{name:"Configure Network".to_string(),id:4,..Default::default()});
    choises.push(MenuChoise{name:"Select Kernel".to_string(),id:5,..Default::default()});
    choises.push(MenuChoise{name:"Print Profile".to_string(),id:6,..Default::default()});
    choises.push(MenuChoise{name:"Build".to_string(),id:7,..Default::default()});
    choises.push(MenuChoise{name:"Confirm".to_string(),id:9,..Default::default()});
    choises.push(MenuChoise{name:"Exit".to_string(),id:8,..Default::default()});

    loop {
        choises = update_validity(&profile, choises);
        let mut options: Vec<String> = vec![];

        for names in &mut choises {
            let mut name_string: String = names.name.clone();
            if names.valid{
                println!("Is valid, adding emoji");
                name_string.push_str(console::Emoji(" ✔️", "").0);
            }
            // println!("{}",name_string);
            options.push(name_string);
        }
        // return;

        /*

        let term = Term::stdout();
        match term.clear_screen() {
            Ok(t) => t,
            Err(error) => panic!("Problem getting terminal: {:?}", error),
        };
        
        */


        let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .items(&options)
            .default(0)
            .interact_on_opt(&Term::stderr());

        let mut choise = 0;
        match selection {
            Ok(value) => choise = value.unwrap(),
            Err(error) => panic!("Problem with selection: {:?}", error),
        };
        match choises[choise].id {
            0 => println!("Network configuration"),
            3 => profile.drives = partition_drives(),
            1 => profile = select_timezone(profile),
            4 => profile.keyboard_layout = select_keyboard_layout(),
            5 => {
                let k:Kernel = Kernel::Linux;
                let mut lst: Vec<Kernel> = vec![];
                let select: usize = select_selection_index_from_strings(get_kernel_strings());
                for ker in Kernel::iter() {
                    if ker.to_string() == get_kernel_strings()[select] {
                        profile.kernel = ker;
                    }
                }
            },
            6 => print_profile(&profile),
            7 => println!("{}",build(&profile)),
            8 => return,
            9 => return,
            _ => println!("Unknown"),
        };
    }
    return;
}

fn build(profile: &Profile) -> String{

    let mut script: String = String::new();
    // Setup build script headers
    script += "#!/bin/sh\n";
    // Set keyboard layout
    script += "loadkeys ";
    script += &profile.keyboard_layout;
    script += "\n";
    script += "timedatectl set-timezone ";
    script += &profile.time_zone;


    // check for space

    // check for EFI partition
    script += "ISEFI = FALSE
    \nif [ -z ($fdisk -l | grep 'EFI') ]\n
    then\n
        echo 'is EFI'\n
        $ISEFI = TRUE\n
    fi
    ";
    // partitions
    for partition in &profile.drives {
        
    }



    return  script;
}

fn main() {
    if !ask_about_new_profile().is_ok() {
        return;
    }
    let mut new_profile: Profile = Default::default();
    options_panel(new_profile);
    return;
    // new_profile.time_zone = select_timezone();
    new_profile.keyboard_layout = select_keyboard_layout();
    println!(
        "\nTimezone: {}\nKeybord layout: {}",
        new_profile.time_zone, new_profile.keyboard_layout
    );

    //new_profile.partitions.push(test_partition);
    new_profile.drives = partition_drives();
    print_profile(&new_profile);

    // !TODO
    // Check network
    // Sync time
    // Setup disks
    // mount file system
    // Package install
    // Pacstrap
    // FSTAB
    // Localegen
    // Network config
    // Kernal modules
    // Grub install
    // EFI?
    // VM?
    // Reboot
}
