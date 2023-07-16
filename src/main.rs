mod unpack;

fn main() -> Result<(), anyhow::Error> {
    let Some(file_name) = std::env::args().nth(1) else {
        eprintln!("Usage: ./unipack path_to.unitypackage");
        return Ok(())
    };

    unpack::unpack(&file_name)
}
