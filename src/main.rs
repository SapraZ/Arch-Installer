use dialoguer;
use std::io;
use console;
use sysinfo;
use block_utils;
enum Kernel {
    linux,
    linuxLTS,
    linuxXanmod,
}
impl std::fmt::Display for Kernel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Kernel::linux => write!(f, "linux"),
            Kernel::linuxLTS => write!(f, "linux-lts"),
            Kernel::linuxXanmod => write!(f, "linux-xanmod"),
        }
    }
}
enum FileSystem {
    ext4,
    btrfs,
}
enum PartitionTypes {
    efi,
    swap,
    root,
    home,
}
struct Partition {
    size: i32,
    partition_type: PartitionTypes,
    file_system: FileSystem,
}

enum DriveType {
    nvme,
    hdd,
}
struct Drive {
    name: String,
    drive_type: DriveType,
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
            kernel: Kernel::linux,
            user_name: "user".to_string(),
            host_name: "ArchLinux".to_string(),
            keyboard_layout: "se-lat6".to_string(),
            time_zone: "UTC+0".to_string(),
            drives: Vec::new(),
        }
    }
}

fn AskAboutNewProfile() -> std::io::Result<()> {
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

fn PrintProfile(profile: &Profile) {
    println!("-=[ Profile ]=-");
    println!("- Kernel: {}", profile.kernel);
    println!("- Drives");
    for drive in &profile.drives {
        println!("  - {}", drive.name);
        for partitions in &drive.partitions {
            println!("      - {} MB", partitions.size.to_string())
        }
    }
}

fn partition_drives() -> Vec<Drive>{

    let mut drives: Vec<Drive>;

    fn get_drives() {
        let block_devices = block_utils::get_block_devices();
        for i in block_devices {
            println!("{}",i);
        }
    }

    fn select_drive(drives:&Vec<Drive>) -> std::io::Result<()> {
        let mut drive_names: Vec<String> = vec![];
        for drive in drives {
            drive_names.push(drive.name.to_string());
        }
        let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .items(&drive_names)
            .default(0)
            .interact_on_opt(& console::Term::stderr())?;

        match selection {
            Some(index) => println!("User selected item : {}", drive_names[index]),
            None => println!("User did not select anything"),
        }

        Ok(())
    }

    fn format_drive() {}


    select_drive(&drives);
    return  drives;

}

fn main() {
    if !AskAboutNewProfile().is_ok() {
        return;
    }
    let mut new_profile: Profile = Default::default();
    let mut new_drive: Drive = Drive {
        name: ("/dev/sda".to_string()),
        drive_type: DriveType::nvme,
        partitions: vec![],
    };
    let new_partition: Partition = Partition {
        size: (200),
        partition_type: PartitionTypes::root,
        file_system: FileSystem::ext4,
    };
    new_drive.partitions.push(new_partition);
    new_profile.drives.push(new_drive);
    //new_profile.partitions.push(test_partition);
    partition_drives();
    PrintProfile(&new_profile);
}
