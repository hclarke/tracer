use std;
use vec::*;
use std::ops::Range;

fn hash(cur : u64, idx : u64) -> u64 {
	let c = 0xf46053d10d8c49f5_u64;	
	let x = idx^cur;
	let x = x^(x>>32);
	let x = x^cur;
	let x = x.wrapping_mul(c);
	let x = x^(x>>32);
	x
}

pub struct Rng(u64);

impl Rng {
	pub fn new(seed : u64) -> Self {
		//shake it up to handle crappy seeds
		let mut v = seed;
		//constants from random.org
		v ^= 0xa0a50dc04975f3e1;
		v = hash(v,0xa09be7c64159c602);
		v = hash(v,0xf2896bf886621af5);
		Rng(v)
	}

	pub fn sub(&self, idx : usize) -> Rng {
		Rng(hash(self.0, (idx as u64) | (2<<57)))
	}

	pub fn u32(&self) -> u32 {
		self.0 as u32
	}

	pub fn uval(&self) -> f32 {
		let r = self.u32();
		let v = (r>>9) | 0x3f800000;
		let f : f32 = unsafe { std::mem::transmute(v) };
		f - 1.0
	}

	pub fn sval(&self) -> f32 {
		let r = self.u32();
		let v = (r>>9) | 0x40000000;
		let f : f32 = unsafe { std::mem::transmute(v) };
		f - 3.0
	}

	pub fn get<T:Rand>(&self) -> T {
		T::gen(&self)
	}
	pub fn range<T:RandRange>(&self, r:Range<T>) -> T {
		T::range(&self, r)
	}

	pub fn ubox<T:V>(&self) ->T {
		V::init(|i| self.sub(i).uval())
	}

	pub fn sbox<T:V>(&self) ->T {
		V::init(|i| self.sub(i).sval())
	}

	pub fn gaussian2(&self) -> V2 {
		let s : V2 = self.ubox();
		let r = (-2.0*s.0.ln()).sqrt();
		let sc = (s.1*2.0*3.14195).sin_cos();
		V2(r*sc.0, r*sc.1)
	}

	pub fn gaussian<T:V>(&self) -> T{
		let dim = T::dim();
		let mut v = T::new(0.0);
		for i in 0..(dim/2) {
			let gr = self.sub(i).gaussian2();
			v[i*2+0] = gr.0;
			v[i*2+1] = gr.1;
		}

		if dim%2 != 0 {
			v[dim-1] = self.sub(dim/2).gaussian2().0;
		}
		v
	}

	pub fn unit<T:RandUnit>(&self) -> T {
		T::unit(&self)
	}
}

pub trait Rand {
	fn gen(rng:&Rng) -> Self;
}
pub trait RandRange : Sized {
	fn range(rng:&Rng, r: Range<Self>) -> Self;
}

impl Rand for u32 {
	fn gen(rng:&Rng) -> Self {
		rng.u32()
	}
}
impl RandRange for u32{
	fn range(rng:&Rng, r: Range<Self>) -> Self {
		let len = r.end-r.start;
		let bins = 0xFFFFFFFF / len;
		let max_valid = bins*len;

		let mut x = 0;
		for i in 0.. {
			x = rng.sub(i).u32();
			if x < max_valid { break; }
		}
		x%len + r.start
	}
}

impl Rand for u64 {
	fn gen(rng:&Rng) -> Self {
		let low = rng.sub(0).u32() as u64;
		let hi = rng.sub(1).u32() as u64;
		low | (hi << 32)
	}
}

impl RandRange for u64 {
	fn range(rng:&Rng, r: Range<Self>) -> Self {
		let len = r.end-r.start;
		let bins = 0xFFFFFFFFFFFFFFFF / len;
		let max_valid = bins*len;

		let mut x : u64 = 0;
		for i in 0.. {
			x = rng.sub(i).get();
			if x < max_valid { break; }
		}
		x%len + r.start
	}
}

impl Rand for usize {
	#[cfg(target_pointer_width = "32")]
	fn gen(rng:&Rng) -> Self {
		rng.u32() as usize
	}

	#[cfg(target_pointer_width = "64")]
	fn gen(rng:&Rng) -> Self {
		let v : u64 = rng.get();
		v as usize
	}
}

impl RandRange for usize {
	#[cfg(target_pointer_width = "32")]
	fn range(rng:&Rng, r:Range<Self>) -> Self {
		rng.range((r.start as u32)..(r.end as u32)) as usize
	}

	#[cfg(target_pointer_width = "64")]
	fn range(rng:&Rng, r:Range<Self>) -> Self {
		rng.range((r.start as u64)..(r.end as u64)) as usize
	}
}

impl Rand for f32 {
	fn gen(rng:&Rng) -> Self {
		rng.uval()
	}
}


pub trait RandUnit {
	fn unit(rng:&Rng) -> Self;
}

impl RandUnit for V3 {
	fn unit(rng:&Rng) -> Self {
		let theta = rng.sub(0).uval() * 2.0*3.14195;
		let (x,y) = theta.sin_cos();
		let r2 = rng.sub(1).uval();
		let r = r2.sqrt();
		let (x,y) = (x*r,y*r);
		let s = (1.0-r2).sqrt();


		V3(2.0*x*s,2.0*y*s,1.0-2.0*r2)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn uval_range() {
		let r = Rng::new(0);
		let count = 1000;
		let mut sum = 0.0;
		for i in 0..count {
			let v = r.sub(i).uval();
			assert!(v >= 0.0);
			assert!(v <= 1.0);
			sum += v;
		}

		//expected value
		let avg = sum/(count as f32);
		assert!((avg-0.5).abs() < 0.01);
	}

	#[test]
	fn sval_range() {
		let r = Rng::new(0);
		let count = 1000;
		let mut sum = 0.0;
		for i in 0..count {
			let v = r.sub(i).sval();
			assert!(v >= -1.0);
			assert!(v <= 1.0);
			sum += v;
		}

		//expected value
		let avg = sum/(count as f32);
		assert!(avg.abs() < 0.01);
	}
}