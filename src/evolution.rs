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
	pub fn genes(&self)->&Vec<Vec<u32>>{&self.genes}
	pub fn models(&self)->&Vec<Graph<Layer<NdArray>>>{&self.models}
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
		let mut results:Vec<Option<(Graph<Layer<NdArray>>,Vec<u32>,f32)>>=(0..size).into_iter().map(|n|{
			filename.clear();

			let _ok=write!(&mut filename,"{directory}/model{n}").inspect_err(|e|println!("failed to generate file name: {e}")).ok()?;

			let _ok=write(&mut BufWriter::new(File::create(&filename).inspect_err(|e|println!("failed to create gene file: {e}")).ok()?),&genes[n]);

			let mut program=Command::new(program).arg(&filename).spawn().inspect_err(|e|println!("failed to start evaluation program: {e}")).ok()?;
			let start=Instant::now();

			loop{
				if let Ok(Some(status))=program.try_wait(){
					if !status.success(){println!("evaluation program didn't exit successfully")}
					break;
				}else{
					if start.elapsed()>*timeout{
						println!("evaluation program timed out");
						program.kill().ok();
					}
					thread::sleep(Duration::from_millis(50));
				}
			}

			from_read(BufReader::new(File::open(&filename).inspect_err(|e|println!("failed to open result file: {e}")).ok()?)).inspect_err(|e|println!("failed to read result file: {e}")).ok()
		}).collect();

		results.extend(models.drain(..).zip(genes.drain(..)).zip(loss.drain(..)).map(|((model,gene),loss)|Some((model,gene,loss))));
		results.sort_unstable_by(|a,b|{
			let (a,b)=(if let Some((_graph,_gene,loss))=a{*loss}else{f32::INFINITY},if let Some((_graph,_gene,loss))=b{*loss}else{f32::INFINITY});
			let (a,b)=(if a.is_nan(){f32::INFINITY}else{a},if b.is_nan(){f32::INFINITY}else{b});

			a.total_cmp(&b)
		});

		for (m,g,l) in results.into_iter().map_while(|x|x).take(capacity){
			genes.push(g);
			loss.push(l);
			models.push(m);
		}
	}
}
#[derive(Clone,Debug,Deserialize,Serialize)]
/// structure for storing a population of models and calling a subprocess to evaluate each one. The subprocess program should accept an argument for a directory to save the model and the evaluation result
pub struct Population{capacity:usize,directory:String,genes:Vec<Vec<u32>>,loss:Vec<f32>,models:Vec<Graph<Layer<NdArray>>>,program:String,threads:usize,timeout:Duration}
use block_graph::{Graph,burn::Layer};
use burn::backend::NdArray;
use crate::gene;
use rmp_serde::{decode::from_read,encode::write};
use serde::{Deserialize,Serialize};
use std::{
	fmt::Write as FmtWrite,fs::{File,create_dir_all},io::{BufReader,BufWriter},process::Command,thread,time::{Duration,Instant}
};
