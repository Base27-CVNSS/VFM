use std::{env, fs, process};

fn usage() -> ! {
    eprintln!("VFM 1.3 reference CLI\n\n  vfm build-minimal <output.vfm>\n  vfm validate <file.vfm>\n  vfm inspect <file.vfm>");
    process::exit(2)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 { usage(); }
    match args[1].as_str() {
        "build-minimal" => {
            let data = vfm_core::write_core_minimal();
            fs::write(&args[2], &data).unwrap_or_else(|e| { eprintln!("write failed: {e}"); process::exit(1) });
            println!("wrote {} bytes sha256={}", data.len(), vfm_core::artifact_sha256(&data));
        }
        "validate" => {
            let data = fs::read(&args[2]).unwrap_or_else(|e| { eprintln!("read failed: {e}"); process::exit(1) });
            match vfm_core::read(&data) {
                Ok(v) => println!("OK VFM {}.{} sections={} profiles={} sha256={}", v.header.format_major, v.header.format_minor, v.directory.len(), v.profiles.len(), vfm_core::artifact_sha256(&data)),
                Err(e) => { eprintln!("{e}"); process::exit(1); }
            }
        }
        "inspect" => {
            let data = fs::read(&args[2]).unwrap_or_else(|e| { eprintln!("read failed: {e}"); process::exit(1) });
            match vfm_core::read(&data) {
                Ok(v) => {
                    println!("VFM {}.{} size={} sections={}",v.header.format_major,v.header.format_minor,data.len(),v.directory.len());
                    for e in v.directory { println!("{} id={} off={} stored={} raw={} flags=0x{:08x} hash_ref={}",e.fourcc(),e.logical_id,e.offset,e.stored_length,e.raw_length,e.flags,if e.hash_ref==vfm_core::NONE_HASH_REF{"NONE".into()}else{e.hash_ref.to_string()}); }
                    println!("profiles={:?}",v.profiles);
                }
                Err(e) => { eprintln!("{e}"); process::exit(1); }
            }
        }
        _ => usage(),
    }
}
