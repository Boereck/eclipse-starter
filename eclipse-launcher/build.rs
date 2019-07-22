use std::io;
use winres::WindowsResource;

fn main() {
    if let Err(err) = set_resource_info() {
        eprintln!("{}", err);
    }
    
}

fn set_resource_info() -> Result<(), io::Error>  {
    if cfg!(target_os = "windows") {
        let mut res = WindowsResource::new();
        res.set_icon("res/eclipse.ico");
        res.compile()?;
    }
    Ok(())
}