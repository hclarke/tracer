use std;
use vec::*;

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

	pub fn sub(&self, idx : usize) -> Rng {
		Rng(hash(self.0, (idx as u64) | (2<<57)))
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

	pub fn unit<T:V>(&self) -> T {
		self.gaussian::<T>().unit()
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