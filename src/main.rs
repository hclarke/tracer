extern crate png_encode_mini;

use std::fs::File;


mod rnd;
mod vec;

use vec::*;
use rnd::*;

fn sd_sphere(p:V3,r:f32) -> f32 {
	p.mag()-r
}

fn scene(p:V3) -> (f32,V3) {
	let d = sd_sphere(p-V3(0.0,0.0,0.0), 1.0);
	let c = V3(0.3,0.5,0.2);
	(d,c)
}

fn sky(d:V3) -> V3 {
	V3::new(1.0)
}

fn scene_normal(p:V3) -> V3 {
	let e = 0.000001;
	let x = scene(p+V3::x()*e).0-scene(p-V3::x()*e).0;
	let y = scene(p+V3::y()*e).0-scene(p-V3::y()*e).0;
	let z = scene(p+V3::z()*e).0-scene(p-V3::z()*e).0;

	V3(x,y,z).unit()
}

struct SceneHit {
	pos : V3,
	normal : V3,
	col : V3,
}
fn trace(o:V3,d:V3)->Option<SceneHit> {
	let mut p = o;
	let mut sd = 1000.0;

	for iter in 0..64 {
		sd = scene(p).0;
		p = p+d*sd;
		if sd < 0.00001 {
			break;
		}
	}

	p = p - d*0.0001;
	let hit = sd < 0.0001;

	if hit {
		Some(SceneHit {
			pos : p,
			normal : scene_normal(p),
			col : scene(p).1
		})
	}
	else {
		None
	}
}

fn trace_bounces(rng : &Rng, mut o:V3,mut d:V3)->V3 {
	let bounces = 10;

	let mut c = V3::new(1.0);
	let mut mult = 1.0;
	for i in 0..bounces {
		match trace(o,d) {
    		Some(SceneHit{pos,normal,col}) => {
    			let l = rng.sub(i).unit();
    			let l = if normal.dot(l) < 0.0 { -l } else { l };
    			c = c * col *  normal.dot(l);
    			o = pos;
    			d = l;
    		},
    		None => {
    			break;
    		}
   		};
	}

	c*sky(d)

}

fn main() {
    let mut file = File::create("out.png").unwrap();

    let width = 256;
    let height = 256;

    let mut frame = Vec::with_capacity(width*height*4);
    frame.resize(width*height*4, 0);

    let cam = V3(0.0,0.0,-2.0);
    let rng = Rng::new(0);
    for j in 0..height {
    	let rng = rng.sub(j);
    	for i in 0..width {
    		let rng = rng.sub(i);
    		let pos = V3(i as f32, j as f32, 0.0) / V3(width as f32, height as f32, 1.0) * 2.0 - 1.0;
    		let dir = (pos-cam).unit();
    		//let col = match trace(cam,dir) {
    		//	Some(SceneHit{pos,normal,col}) => {
    		//		col
    		//	},
    		//	None => V3::new(0.0)
    		//;

    		let mut col = V3::new(0.0);
    		let ray_count = 100;
    		for k in 0..ray_count {
    			let rng = rng.sub(k);
    			col = col + trace_bounces(&rng, pos,dir) / (ray_count as f32);
    		}
    		frame[(j*width+i)*4+0] = (col.0 * 256.0).min(255.0).max(0.0) as u8;
    		frame[(j*width+i)*4+1] = (col.1 * 256.0).min(255.0).max(0.0) as u8;
    		frame[(j*width+i)*4+2] = (col.2 * 256.0).min(255.0).max(0.0) as u8;
    		frame[(j*width+i)*4+3] = 255; 
    	}
    }


    png_encode_mini::write_rgba_from_u8(&mut file, &frame[..], width as u32,height as u32).unwrap();
}
