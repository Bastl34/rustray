use std::env;
//use std::path::Path;
use std::path::PathBuf;

fn main()
{
    if let Err(_) = env::var("OIDN_DIR")
    {
        println!("cargo:error=Please set OIDN_DIR=<path to OpenImageDenoise install root>");
        panic!("Failed to find OpenImageDenoise");
    }

    println!("cargo:rerun-if-env-changed=OIDN_DIR");

    //copy oidn libs
    let mut oidn_dir = PathBuf::from(env::var("OIDN_DIR").unwrap());
    oidn_dir.push("lib");

    //let output_path = get_output_path();

    let files = 
    [
        "libOpenImageDenoise.1.4.2.dylib",
        "libOpenImageDenoise.1.dylib",
        "libOpenImageDenoise.dylib",
        "libtbb.12.4.dylib",
        "libtbb.12.dylib",
        "libtbb.dylib",
    ];

    for file in files
    {
        let from = format!("{}/{}", oidn_dir.display(), file);
        let to = format!("{}/../../../deps/{}", env::var("OUT_DIR").unwrap(), file);
        std::fs::copy(&from, to).unwrap();
    }
}

/*
fn get_output_path() -> PathBuf
{
    //<root or manifest path>/target/<profile>/
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string).join("target").join(build_type);
    return PathBuf::from(path);
}
*/