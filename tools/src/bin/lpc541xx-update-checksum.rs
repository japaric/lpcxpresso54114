#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]

use std::{env, error::Error, fs, path::PathBuf, process};

use xmas_elf::ElfFile;

fn main() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(env::args().nth(1).unwrap());
    let bytes = fs::read(&path)?;
    let ef = ElfFile::new(&bytes)?;

    if let Some(sect) = ef.find_section_by_name(".vectors") {
        let data = sect.raw_data(&ef);

        unsafe {
            update(data.as_ptr() as *mut u8);
        }

        fs::write(path, bytes)?;
    } else {
        eprintln!("error: no `.vectors` section found");

        process::exit(1);
    }

    Ok(())
}

unsafe fn update(start: *mut u8) {
    let mut p = start as *mut [u8; 4];

    let mut sum = 0u32;
    for _ in 0..7 {
        sum += u32::from_le_bytes(*p);
        p = p.add(1);
    }

    let checksum = (!sum) + 1;
    println!("checksum: {:#010x}", checksum);
    (*p) = checksum.to_le_bytes()
}
