#![recursion_limit = "256"]
// example to help get started
fn main(){
	let model=mnist::basic_mnist_model();
	let model=mnist::train_model(model);

	mnist::test_model(&model);

	//data::save_model(&model,"m0").unwrap();	// block graph TODO s: implement (de)serialize for Graph, add flatten and reshape layers
}
pub mod evolution;
/// load, save, and other data related utilities
pub mod data;
pub mod gene;
/// mnist example and utilities
pub mod mnist;
