#![recursion_limit = "256"]
fn main(){
	let args:Vec<String>=env::args().collect();
	let path=args.get(1);

	if let Some(pathtoevaluate)=path{
		evaluate_test(pathtoevaluate);
	}else{
		let mut population=Population::from_models("evolve lowest average token test",vec![Default::default()],"./block-evolution",Duration::from_secs_f32(100.0));
		for _n in 0..10{
			population.run_generation();
			dbg!(&population.genes()[0]);
		}
	}
}
/// test the evolution framework by evolving a gene sequence to have the lowest average token
pub fn evaluate_test(path:&str){
	let gene:Vec<u32>=data::load_model(path).unwrap();
	let gene=gene::mutate(gene,0.05,0.05,0.1);
	let model=gene::build_model(&gene);
	let mut loss:f32=gene.iter().map(|&x|x as f32).sum();

	loss/=gene.len() as f32;

	data::save_model(&(model,gene,loss),path).unwrap();
}

pub mod evolution;
/// load, save, and other data related utilities
pub mod data;
pub mod gene;
/// mnist example and utilities
pub mod mnist;
use evolution::Population;
use std::{env,time::Duration};
