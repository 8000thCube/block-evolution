pub fn load_model<P:Into<PathBuf>,T:DeserializeOwned>(path:P)->IOResult<T>{
	let file=File::open(path.into())?;
	let reader=BufReader::new(file);

	Ok(decode::from_read(reader).unwrap())
}
pub fn save_model<P:Into<PathBuf>,T:Serialize>(model:&T,path:P)->IOResult<()>{
	let path=path.into();

	let mut writer=BufWriter::new(File::create(&path)?);
	Ok(encode::write(&mut writer,model).unwrap())
}
use rmp_serde::{decode,encode};
use std::{
	fs::File,io::{BufReader,BufWriter,Result as IOResult},path::PathBuf
};
use serde::{Serialize,de::DeserializeOwned};
