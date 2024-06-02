use std::fs;
use std::path::PathBuf;

//fn convert() {}

pub fn convert_to_file(
	path: &PathBuf,
	output_file: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
	if path.file_name().unwrap().to_str().unwrap().starts_with('_') {
		return Ok(());
	}

	//if let Some(file_name) = path.file_name() {
	//	if let Some(file_name_str) = file_name.to_str() {
	//		println!("str: {file_name_str}");
	//		if file_name_str.starts_with('_') {
	//			return Ok(());
	//		}
	//	}
	//}

	println!("{path:?}:{output_file:?}");

	// Create dir if it doesn't exist
	let parent = match output_file.parent() {
		Some(dir) => dir,
		None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Could not find parent Directory"))),
	};

	if !parent.exists() {
		fs::create_dir_all(&parent)?;
	}

	fs::write(
		output_file,
		grass::from_path(path, &grass::Options::default())?,
	)?;

	Ok(())
}
