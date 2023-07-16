pub mod unpack;

pub fn unpack(file_name: &str) -> anyhow::Result<()> {
	unpack::unpack(file_name)
}
