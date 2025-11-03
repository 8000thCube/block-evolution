// mnist example and utilities

// returns a basic model with a couple linear and relu layers. by default this outputs a logits distribution, apply .soft_choose to get a model that gives numeric guesses
pub fn basic_mnist_model()->Graph<Layer<Autodiff<NdArray>>>{
	let mut graph:Graph<Layer<Autodiff<NdArray>>>=Graph::new();

	graph.connect("input","x").with_clear(true).with(Layer::flatten(-2..=-1));
    graph.connect("x",1_usize).with_clear(true).with(Layer::linear(true,28*28,64,1.0));
	graph.connect(1_usize,2_usize).with_clear(true).with(Layer::relu());
	graph.connect(2_usize,3_usize).with_clear(true).with(Layer::linear(true,64,32,1.0));
	graph.connect(3_usize,4_usize).with_clear(true).with(Layer::relu());
	graph.connect(4_usize,"y").with_clear(true).with(Layer::linear(false,32,10,1.0));

    graph
}
/// tests the model, returning the accuracy
pub fn evaluate_model<B:Backend>(examplecount:usize,graph:&Graph<Layer<B>>)->f32{
    let graph=Unvec(graph).soft_choose(1.0);
    let testdata=MNIST::load_validation_data();

	let mut k=0;
	for n in 0..examplecount{
		let (testinput,testtarget)=testdata.get(n).unwrap();
		let expectedoutput=testtarget.into_float_vec()[0] as u32;
		let testoutput:u32=graph.forward(testinput.reshape([28_isize,28]));

		if expectedoutput==testoutput{k+=1}
	}
	k as f32/examplecount as f32*100.0
}
/// reads the loss from the .artifact directory
pub fn read_loss(epoch:usize)->IOResult<f32>{
	let file=std::fs::read_to_string(&format!(".artifact/epoch-{epoch}/Loss.log"))?;
	let lines=file.split('\n');

	todo!()
	//lines.map(|s|s.)
}
/// trains the model
pub fn train_model<B:Backend>(graph:Graph<Layer<Autodiff<B>>>)->Graph<Layer<B>>{
    let batch=32;
	let epochs=3;
	let lr=0.001;
	let optimizer=AdamWConfig::new().init();
	let train=MNIST::load_training_data();
	let trainconfig=TrainConfig::new().with_batch_size(batch).with_epochs(epochs).with_console_rendering(true);
	let trainconfig=&trainconfig;
	let valid=MNIST::load_validation_data();

	let graph=Unvec(graph).wrap_inner().cross_entropy(1.0).set_type::<(Value<Autodiff<B>>,Value<Autodiff<B>>),LossOutput<Autodiff<B>>>().classification().wrap();
    let graph=graph.train(trainconfig,optimizer,lr,train,valid);
	let graph=graph.valid().unwrap_inner();

    graph.0
}
/// tests the model
pub fn test_model(graph:&Graph<Layer<NdArray>>){
    let graph=Unvec(graph).soft_choose(1.0);
    let testdata=MNIST::load_validation_data();

	let mut k=0;
	for n in 0..100{
		let (testinput,testtarget)=testdata.get(n).unwrap();
		let expectedoutput=testtarget.into_float_vec()[0] as u32;
		let testoutput:u32=graph.forward(testinput.reshape([28_isize,28]));

		if expectedoutput==testoutput{k+=1}
		println!("expected {expectedoutput}, output {testoutput}");
	}
	println!("accuracy {k}%");
}
impl MNIST{
	/// loads the mnist dataset
	pub fn load_training_data()->Self{
		Self{inner:MnistDataset::train()}
	}
	/// loads the mnist dataset
	pub fn load_validation_data()->Self{
		Self{inner:MnistDataset::test()}

	}
}
impl<B:Backend> Dataset < (Value<B>, Value<B>) > for MNIST { // (image, label)
	fn get(&self, index:usize) -> Option <  (Value<B>, Value<B>) > {
		let data = self.inner.get(index)?;
		let image:Vec<f32> = data.image.into_iter().flat_map(|row|row).collect();
		let label = vec![data.label as i32];

		let input = Value::from(image);
		let target = Value::from(label);

		let input = input.reshape([1_usize,28,28]);

		Some((input, target))
	}

	fn len (&self) -> usize {
		return self.inner.len()
	}
}
struct MNIST{inner:MnistDataset}
use block_graph::{
	AI,Graph,Op,Unvec,UnwrapInner,burn::{Layer,LossOutput,Shortcuts,TrainConfig,Value},ops::Reshape
};
use burn::{
	backend::{Autodiff,NdArray},data::dataset::{Dataset,vision::MnistDataset},module::AutodiffModule,optim::AdamWConfig,prelude::Backend
};
use std::{fs::read_to_string,io::Result as IOResult};
