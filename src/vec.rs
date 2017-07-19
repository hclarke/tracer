use std::ops::*;

pub trait V : Sized 
	+ Copy
	+ Neg<Output=Self>
	+ Add<Self,Output=Self> 
	+ Sub<Self,Output=Self> 
	+ Mul<Self,Output=Self> 
	+ Div<Self,Output=Self> 
	+ Add<f32,Output=Self> 
	+ Sub<f32,Output=Self> 
	+ Mul<f32,Output=Self> 
	+ Div<f32,Output=Self>
	+ Index<usize,Output=f32>
	+ IndexMut<usize,Output=f32> {

	fn dim() -> usize;
	fn new(v:f32)->Self;
	fn init<F>(f:F)->Self where F:Fn(usize)->f32;
	fn map<F>(self, f:F) -> Self where F:Fn(f32)->f32;
	fn zip<F>(self, rhs:Self, f:F) -> Self where F:Fn(f32,f32)->f32;
	fn foldr<F,T>(self, v:T,f:F) -> T where F:Fn(f32,T)->T;

	fn sum(self) -> f32 {
		self.foldr(0.0,|a,b|a+b)
	}
	fn prod(self) -> f32 {
		self.foldr(1.0,|a,b|a*b)
	}
	fn dot(self,rhs:Self) -> f32 {
		(self*rhs).sum()
	}
	fn sqr_mag(self) -> f32 {
		self.dot(self)
	}
	fn mag(self) -> f32 {
		self.sqr_mag().sqrt()
	}
	fn unit(self) -> Self {
		self/self.mag()
	}
}

macro_rules! impl_vector_ops {
	($T:ty) => {
		impl Neg for $T {
			type Output = Self;
			fn neg(self) -> Self {
				self.map(|x| -x)
			}
		}

		impl Add for $T {
			type Output = Self;
			fn add(self, rhs : Self) -> Self {
				self.zip(rhs, |a,b| a+b)
			}
		}
		
		impl Sub for $T {
			type Output = Self;
			fn sub(self, rhs : Self) -> Self {
				self.zip(rhs, |a,b| a-b)
			}
		}
		
		impl Mul for $T {
			type Output = Self;
			fn mul(self, rhs : Self) -> Self {
				self.zip(rhs, |a,b| a*b)
			}
		}
		
		impl Div for $T {
			type Output = Self;
			fn div(self, rhs : Self) -> Self {
				self.zip(rhs, |a,b| a/b)
			}
		}
		
		impl Add<f32> for $T {
			type Output = Self;
			fn add(self, rhs : f32) -> Self {
				self.map(|a| a+rhs)
			}
		}
		
		impl Sub<f32> for $T {
			type Output = Self;
			fn sub(self, rhs : f32) -> Self {
				self.map(|a| a-rhs)
			}
		}
		
		impl Mul<f32> for $T {
			type Output = Self;
			fn mul(self, rhs : f32) -> Self {
				self.map(|a| a*rhs)
			}
		}
		
		impl Div<f32> for $T {
			type Output = Self;
			fn div(self, rhs : f32) -> Self {
				self.map(|a| a/rhs)
			}
		}
	}
}

macro_rules! impl_foldr {
    ($target:ident, $f:ident, $v:ident, $fst:tt, $($i:tt),+) => ($f($target.$fst, impl_foldr!($target,$f,$v,$($i),+)));
    ($target:ident, $f:ident, $v:ident, $fst:tt) => ($f($target.$fst,$v));
}

macro_rules! fst_expr {
	($e:expr,$snd:tt) => {$e}
}

macro_rules! count {
	($fst:tt, $($rst:tt),+) => (1+count!($($rst),+));
	($fst:tt) => (1);
}

macro_rules! impl_vector {
	($T:tt, $($idx:tt),+) => {
		impl_vector_ops!($T);
		impl V for $T {
			fn dim() -> usize {
				count!($($idx),+)
			}
			fn new(v:f32)->Self {
				$T(
					$(fst_expr!(v,$idx)),+
				)
			}
			fn init<F>(f:F)->Self where F:Fn(usize)->f32 {
				$T(
					$(f($idx)),+
				)
			}
			fn map<F>(self, f:F) -> Self where F:Fn(f32)->f32 {
				$T(
					$(f(self.$idx)),+
				)
			}
			fn zip<F>(self, rhs:Self, f:F) -> Self where F:Fn(f32,f32)->f32 {
				$T(
					$(f(self.$idx,rhs.$idx)),+
				)
			}
			fn foldr<F,T>(self, v:T,f:F) -> T where F:Fn(f32,T)->T {
				impl_foldr!(self,f,v, $($idx),+)
			}
		}


		impl Index<usize> for $T {
			type Output = f32;
			fn index(&self, i:usize) -> &f32 {
				match i {
					$($idx => &self.$idx,)+
					_ => panic!(),
				}
			}
		}

		impl IndexMut<usize> for $T {
			fn index_mut(&mut self, i:usize) -> &mut f32 {
				match i {
					$($idx => &mut self.$idx,)+
					_ => panic!(),
				}
			}
		}
	}
}


#[derive(Copy,Clone,Debug)]
pub struct V2(pub f32,pub f32);
impl_vector!(V2, 0,1);

#[derive(Copy,Clone,Debug)]
pub struct V3(pub f32,pub f32,pub f32);
impl_vector!(V3, 0,1,2);

#[derive(Copy,Clone,Debug)]
pub struct V4(pub f32,pub f32,pub f32, pub f32);
impl_vector!(V4, 0,1,2,3);

impl V3 {
	pub fn x() -> Self {
		V3(1.0,0.0,0.0)
	}
	pub fn y() -> Self {
		V3(0.0,1.0,0.0)
	}
	pub fn z() -> Self {
		V3(0.0,0.0,1.0)
	}
	pub fn cross(self,rhs:V3) -> V3 {
		V3(
			self.1*rhs.2-self.2*rhs.1,
			self.2*rhs.0-self.0*rhs.2,
			self.0*rhs.1-self.1*rhs.0,
		)
	}
}
