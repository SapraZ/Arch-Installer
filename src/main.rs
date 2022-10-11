use std::io;
use dialoguer;

enum Kernel {
    linux,
    linuxLTS,
    linuxXanmod
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
    btrfs
}
enum PartitionTypes {
    efi,
    swap,
    root,
    home
}
struct Partition {
    drive: String,
    size: i32,
    partition_type: PartitionTypes,
    file_system: FileSystem
}

struct Profile {
    partitions: Vec<Partition>,
    kernel: Kernel,
    time_zone: String,
    keyboard_layout: String,
    host_name: String,
    user_name: String
}

impl Default for Profile {
    fn default() -> Profile {
        Profile {
            kernel:Kernel::linux,
            user_name: "user".to_string(),
            host_name: "ArchLinux".to_string(),
            keyboard_layout: "se-lat6".to_string(),
            time_zone: "UTC+0".to_string(),
            partitions: Vec::new()
        }
   }
}

fn AskAboutNewProfile() -> std::io::Result<()> {
    println!("Welcome to ArchInstaller 0.1");
    if dialoguer::Confirm::new().with_prompt("Do you want to create a new profile?").interact()? {
        println!("Looks like you want to continue");
    } else {
        println!("nevermind then :(");
    }
    return Ok(())
}

fn PrintProfile(profile:&Profile) {
    println!("--- Profile ---");
    println!("- Kernel: {}",profile.kernel);
    for drive in &profile.partitions{
        println!("- {}",drive.drive);
        for files_system in drive.file_system {
            
        }
    }
}

fn main() {
    if !AskAboutNewProfile().is_ok() {
        return;
    }
    let mut new_profile: Profile = Default::default();
    let test_partition: Partition = Partition { drive: ("/dev/sda".to_string()), size: (128), partition_type: (PartitionTypes::root), file_system: (FileSystem::ext4) };
    new_profile.partitions.push(test_partition);
    PrintProfile(&new_profile);
}
