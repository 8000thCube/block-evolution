pub const DEFAULT_CAPACITY:usize=100;
impl Population{
	/// creates a new population from a directory, genes, evaluation program, and a time after which to kill the evaluation subprocess if it takes too long. The program should accept an argument for a directory to save the model and evaluation result as (Graph<Layer<..>>,f32), in rmp serde format
	pub fn from_genes<S:AsRef<str>,T:AsRef<str>>(directory:S,genes:Vec<Vec<u32>>,program:T,timeout:Duration)->Self{
		let (directory,program)=(directory.as_ref().to_string(),program.as_ref().to_string());
		let (loss,models)=(Vec::new(),Vec::new());
		let capacity=DEFAULT_CAPACITY;

		Self{capacity,directory,genes,loss,models,program,timeout}
	}
	/// creates a new population from a directory, models, evaluation program, and a time after which to kill the evaluation subprocess if it takes too long. The program should accept an argument for a directory to save the model and evaluation result as (Graph<Layer<..>>,f32), in rmp serde format
	pub fn from_models<S:AsRef<str>,T:AsRef<str>>(directory:S,models:Vec<Graph<Layer<NdArray>>>,program:T,timeout:Duration)->Self{
		let (directory,program)=(directory.as_ref().to_string(),program.as_ref().to_string());
		let (genes,loss)=(Vec::new(),Vec::new());
		let capacity=DEFAULT_CAPACITY;

		Self{capacity,directory,genes,loss,models,program,timeout}
	}
	/// runs a generation of models
	pub fn run_generation(&mut self){
		let (directory,program,timeout)=(&self.directory,&self.program,&self.timeout);
		let (genes,loss,models)=(&mut self.genes,&mut self.loss,&mut self.models);
		let (gl,ml)=(genes.len(),models.len());
		let capacity=self.capacity;
		let size=gl.max(ml);

		if gl<ml{
			let mut models=models.iter();
			genes.resize_with(ml,||gene::transcribe_gene(models.next().unwrap()));
		}else if gl>ml{
			let mut genes=genes.iter();
			models.resize_with(ml,||gene::build_model(genes.next().unwrap()));
		}
		loss.resize(size,f32::INFINITY);



		todo!()
	}
}
#[derive(Clone,Debug,Deserialize,Serialize)]
/// structure for storing a population of models and calling a subprocess to evaluate each one. The subprocess program should accept an argument for a directory to save the model and the evaluation result
pub struct Population{capacity:usize,directory:String,genes:Vec<Vec<u32>>,loss:Vec<f32>,models:Vec<Graph<Layer<NdArray>>>,program:String,timeout:Duration}
use block_graph::{
	AI,Graph,Op,Unvec,UnwrapInner,burn::{Layer,LossOutput,Shortcuts,TrainConfig,Value,Wrappable}
};
use burn::{
	backend::{Autodiff,NdArray},data::dataset::{Dataset,vision::MnistDataset},module::AutodiffModule,optim::AdamWConfig,prelude::Backend
};
use crate::gene;
use serde::{Deserialize,Serialize};
use std::{
	process::Command,time::Duration
};
