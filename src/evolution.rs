pub const DEFAULT_CAPACITY:usize=100;
impl Population{
	/// creates a new population from a directory, genes, evaluation program, and a time after which to kill the evaluation subprocess if it takes too long. The program should accept an argument for a file to load the gene as Vec<u32> and to save the model and evaluation result as (Graph<Layer<..>>,Vec<u32>,f32), in rmp serde format
	pub fn from_genes<S:AsRef<str>,T:AsRef<str>>(directory:S,genes:Vec<Vec<u32>>,program:T,timeout:Duration)->Self{
		let (directory,program)=(directory.as_ref().to_string(),program.as_ref().to_string());
		let (loss,models)=(Vec::new(),Vec::new());
		let capacity=DEFAULT_CAPACITY;
		let threads=1;

		Self{capacity,directory,genes,loss,models,program,threads,timeout}
	}
	/// creates a new population from a directory, models, evaluation program, and a time after which to kill the evaluation subprocess if it takes too long. The program should accept an argument for a file to load the gene as Vec<u32> and to save the model and evaluation result as (Graph<Layer<..>>,Vec<u32>,f32), in rmp serde format
	pub fn from_models<S:AsRef<str>,T:AsRef<str>>(directory:S,models:Vec<Graph<Layer<NdArray>>>,program:T,timeout:Duration)->Self{
		let (directory,program)=(directory.as_ref().to_string(),program.as_ref().to_string());
		let (genes,loss)=(Vec::new(),Vec::new());
		let capacity=DEFAULT_CAPACITY;
		let threads=1;

		Self{capacity,directory,genes,loss,models,program,threads,timeout}
	}
	/// runs a generation of models
	pub fn run_generation(&mut self){
		let (directory,program,timeout)=(&self.directory,&self.program,&self.timeout);
		let (genes,loss,models)=(&mut self.genes,&mut self.loss,&mut self.models);
		let (gl,ml)=(genes.len(),models.len());
		let capacity=self.capacity;
		let size=gl.max(ml);

		create_dir_all(directory).expect("must be able to create the directory");
		if gl<ml{
			let mut models=models.iter();
			genes.resize_with(ml,||gene::transcribe_gene(models.next().unwrap()));
		}else if gl>ml{
			let mut genes=genes.iter();
			models.resize_with(ml,||gene::build_model(genes.next().unwrap()));
		}
		loss.resize(size,f32::INFINITY);

		let mut filename=String::with_capacity(directory.len()+10);
		let results:Vec<Option<(Graph<Layer<NdArray>>,Vec<Vec<u32>>,f32)>>=(0..size).into_iter().map(|n|{
			filename.clear();

			let _ok=write!(&mut filename,"{directory}/model{n}").inspect_err(|e|println!("failed to generate file name: {e}")).ok()?;

			let _ok=write(&mut BufWriter::new(File::create(&filename).inspect_err(|e|println!("failed to create gene file: {e}")).ok()?),&genes[n]);

			let status=Command::new(program).arg(&filename).output().map(|status|status.status.success()).inspect_err(|e|println!("failed to evaluate model: {e}")).ok()?;
			from_read(BufReader::new(File::open(&filename).inspect_err(|e|println!("failed to open result file: {e}")).ok()?)).inspect_err(|e|println!("failed to read result file: {e}")).ok()
		}).collect();

		todo!()
	}
}
#[derive(Clone,Debug,Deserialize,Serialize)]
/// structure for storing a population of models and calling a subprocess to evaluate each one. The subprocess program should accept an argument for a directory to save the model and the evaluation result
pub struct Population{capacity:usize,directory:String,genes:Vec<Vec<u32>>,loss:Vec<f32>,models:Vec<Graph<Layer<NdArray>>>,program:String,threads:usize,timeout:Duration}
use block_graph::{
	AI,Graph,Op,Unvec,UnwrapInner,burn::{Layer,LossOutput,Shortcuts,TrainConfig,Value,Wrappable}
};
use burn::{
	backend::{Autodiff,NdArray},data::dataset::{Dataset,vision::MnistDataset},module::AutodiffModule,optim::AdamWConfig,prelude::Backend
};
use crate::gene;
use rmp_serde::{decode::from_read,encode::write};
use serde::{Deserialize,Serialize};
use std::{
	fmt::Write as FmtWrite,fs::{File,create_dir_all},mem,io::{BufReader,BufWriter,Read,Write as IOWrite},process::Command,time::Duration
};
