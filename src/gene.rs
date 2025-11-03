impl Default for GeneEntry{
	fn default()->Self{Self::Invalid(Vec::new())}
}
impl GeneEntry{
	/// converts the gene entry into a raw token sequence
	pub fn into_raw<M:Into<Option<Vec<u32>>>>(self,mem:M)->Vec<u32>{// TODO make proper constants for the magic numbers
		let mut mem=mem.into().unwrap_or_default();
		match self{
			Self::Connection(c)=>{
				mem.push(256);
				mem=label_to_raw(c.input,mem);
				mem.push(' ' as u32);
				mem=label_to_raw(c.label,mem);
				mem.push(' ' as u32);
				mem=label_to_raw(c.layer,mem);
				mem.push(' ' as u32);
				mem=label_to_raw(c.output,mem);
				mem.push(';' as u32);
				mem.push('\n' as u32);
			},
			Self::Invalid(i)=>{
				mem.extend(i.iter().copied());
				if i.last().is_some()&&i.last().copied()!=Some('\n' as u32){mem.push('\n' as u32)}
			},
			Self::Layer(l)=>{
				mem.push(257);
				mem.extend(l.raw_dims());
				mem.push(' ' as u32);
				mem=label_to_raw(l.label,mem);
				mem.push(' ' as u32);
				mem.push(l.variant);
				mem.push(';' as u32);
				mem.push('\n' as u32);
			},
			Self::Node(n)=>{
				mem.push(258);
				mem=label_to_raw(n.label,mem);
				mem.push(';' as u32);
				mem.push('\n' as u32);
			},
			Self::Order(o)=>{
				mem.push(259);
				mem=label_to_raw(o.connectionlabel,mem);
				mem.push(';' as u32);
				mem.push('\n' as u32);
			}
		}
		mem
	}
}
impl GeneSequence{
	/// converts from gene sequence to raw tokens
	pub fn into_raw<M:Into<Option<Vec<u32>>>>(self,mem:M)->Vec<u32>{
		let mem=mem.into().unwrap_or_else(||Vec::with_capacity(self.parsed.len()*10));
		self.parsed.into_iter().fold(mem,|acc,g|g.into_raw(acc))
	}
}
impl ConnectionEntry{
	pub fn from_raw(gene:&[u32])->Self{
		todo!()
	}
}
impl LayerEntry{
	pub fn normalized_dims(&self)->Vec<usize>{
		if self.normalizeddims.len()>0||self.rawdims.len()==0{return self.normalizeddims.clone()}

		let mut acc=0;
		let mut number=0;
		let mut result=Vec::with_capacity(self.rawdims.len()/3);

		self.rawdims.iter().copied().filter_map(char::from_u32).chain([' ']).for_each(|c|if c.is_alphabetic(){
			acc+=number;
			number=0;
		}else if c.is_digit(10){
			number=number*10+c as u32-'0' as u32;
		}else{
			if number>0{
				result.push((acc+number) as usize);
				(acc,number)=(0,0);
			}
		});
		result
	}
	pub fn raw_dims(&self)->Vec<u32>{
		if self.rawdims.len()>0||self.normalizeddims.len()==0{return self.rawdims.clone()}
		let mut result=Vec::with_capacity(self.normalizeddims.len()*3);

		self.normalizeddims.iter().for_each(|n|result.extend(n.to_string().chars().map(|c|c as u32).chain([' ' as u32])));
		result
	}
}
// allowed tokens in the "gene" sequence. This list is not final; we might add specific tokens for layer types
pub const ALLOWED_TOKENS:[u32;43]=[
	'0' as u32,// digits
	'1' as u32,
	'2' as u32,
	'3' as u32,
	'4' as u32,
	'5' as u32,
	'6' as u32,
	'7' as u32,
	'8' as u32,
	'9' as u32,
	'A' as u32,// letters
	'B' as u32,
	'C' as u32,
	'D' as u32,
	'E' as u32,
	'F' as u32,
	'G' as u32,
	'H' as u32,
	'I' as u32,
	'J' as u32,
	'K' as u32,
	'L' as u32,
	'M' as u32,
	'N' as u32,
	'O' as u32,
	'P' as u32,
	'Q' as u32,
	'R' as u32,
	'S' as u32,
	'T' as u32,
	'U' as u32,
	'V' as u32,
	'W' as u32,
	'X' as u32,
	'Y' as u32,
	'Z' as u32,
	' ' as u32,	// space for separator
	';' as u32,	// semicolon for stop codon
	'\n' as u32,// line break. ignored
	256,		// begin connection
	257,		// begin layer
	258,		// begin node
	259,		// begin order
];
#[derive(Clone,Debug,Eq,Hash,PartialEq)]
/// enumeration of types of gene entries
pub enum GeneEntry{Connection(ConnectionEntry),Invalid(Vec<u32>),Layer(LayerEntry),Node(NodeEntry),Order(OrderEntry)}
/// builds a model from the gene
pub fn build_model(_gene:&[u32])->Graph<Layer<NdArray>>{
	// TODO
	Default::default()
}
/// convenience function to convert gene sequence to human readable text
pub fn gene_to_string(gene:&[u32])->String{
	let tokenizer=generate_token_dict();
	tokenizer.detokenize_str(gene).collect()
}
///  generates a dictionary of tokens used to convert between token array and human readable string formats
pub fn generate_token_dict()->TokenDict{
	["connection: ","layer","node:","order:"].into_iter().collect()
}
pub fn label_to_raw<M:Into<Option<Vec<u32>>>>(label:Label,mem:M)->Vec<u32>{
	let mut mem=mem.into().unwrap_or_default();
	mem.extend(label.to_string().chars().map(|c|c as u32));

	mem
}
pub fn mutation_test(){
	let mut gene:Vec<u32>=vec!['H','E','L','L','O',' ','W','O','R','L','D',';'].into_iter().map(|x|x as u32).collect();
	for _ in 0..10{
		for c in gene.iter().map(|&c|char::from_u32(c).unwrap()){print!("{c}")}
		gene=mutate(gene,0.05,0.05,0.1);
		println!();
	}
}
/// at each position in the gene, possibly apply the three types of point mutations according to their respective probabilities	// TODO although this function will have a relatively low impact on performance compared to training, it could be optimized
pub fn mutate(mut gene:Vec<u32>,
              deletionchance:f32,
              insertionchance:f32,
              substitutionchance:f32
             ) ->Vec<u32>{
    let mut rng = rand::rng();
	use rand::Rng;
    use rand::seq::IndexedRandom;
    let mut y = 0;

    while y < gene.len() {
        let mut x: f32 = rng.random();
        if x < deletionchance {
            gene.remove(y);

        }

        x = rng.random();
        if x < insertionchance {
            let token = *ALLOWED_TOKENS.choose(&mut rng).unwrap();
            gene.insert(y, token);
            y = y + 1;
        }

        x = rng.random();
        if x < substitutionchance &&y<gene.len(){
            gene[y] = *ALLOWED_TOKENS.choose(&mut rng).unwrap();
        }

		y = y + 1;
    }

    gene
}
/// generates a gene that produces the model structure
pub fn transcribe_gene(_model:&Graph<Layer<NdArray>>)->Vec<u32>{
	TokenDict::default().string_to_tokens("TEST")
}
/// returns true with probability chance
pub fn should_mutate(chance:f32)->bool{
	let choice:f32=rand::random();
	choice<chance
}
#[derive(Clone,Debug,Default,Eq,Hash,PartialEq)]
/// connection entry
pub struct ConnectionEntry{input:Label,label:Label,layer:Label,output:Label}
#[derive(Clone,Debug,Default,Eq,Hash,PartialEq)]
/// gene sequence structure
pub struct GeneSequence{parsed:Vec<GeneEntry>}
#[derive(Clone,Debug,Default,Eq,Hash,PartialEq)]
/// layer entry
pub struct LayerEntry{label:Label,normalizeddims:Vec<usize>,rawdims:Vec<u32>,variant:u32}
#[derive(Clone,Debug,Default,Eq,Hash,PartialEq)]
/// node entry
pub struct NodeEntry{label:Label}
#[derive(Clone,Debug,Default,Eq,Hash,PartialEq)]
/// order entry
pub struct OrderEntry{connectionlabel:Label}
use block_graph::{Graph,Label,burn::Layer};
use burn::backend::NdArray;
use token_dict::TokenDict;
