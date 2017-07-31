extern crate png_encode_mini;

use std::fs::File;


mod rnd;
mod vec;

use vec::*;
use rnd::*;

fn sd_sphere(p:V3,r:f32) -> f32 {
	p.mag()-r
}

fn union<T>(a:(f32,T), b:(f32,T)) -> (f32,T) {
	if a.0 < b.0 {
		a
	}
	else {
		b
	}
}

fn scene(p:V3) -> (f32,V3) {
	let s0 = sd_sphere(p-V3(1.0,0.0,0.0), 1.0);
	let c0 = V3(0.3,0.5,0.2);

	let o0 = (s0,c0);


	let s1 = sd_sphere(p-V3(-0.75,0.0,0.0), 0.75);
	let c1 = V3(0.8,0.2,0.2);

	let o1 = (s1,c1);


	union(o0, o1)
}

fn sky(d:V3) -> V3 {
	let _ = d;
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
fn trace(o:V3,d:V3, max_dist:f32)->Option<SceneHit> {
	let mut dist = 0.0;
	let mut sd = 0.0;

	for _ in 0..64 {
		sd = scene(o+d*dist).0;
		dist += sd;
		if dist > max_dist || sd < 0.00001 {
			break;
		}
	}

	dist -= 0.0001;
	let hit = sd < 0.01;
	let p = o+d*dist;
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
	for i in 0..bounces {
		match trace(o,d, 1000.0) {
    		Some(SceneHit{pos,normal,col}) => {
    			let l = rng.sub(i).unit();
    			let l = if normal.dot(l) < 0.0 { -l } else { l };
    			c = c * col *  normal.dot(l);
    			o = pos;
    			d = l;
    		},
    		None => {
    			return c*sky(d);
    		}
   		};
	}

	V3::new(0.0)
}

fn monte_carlo(rng : &Rng, o:V3, d:V3) -> V3 {
	let mut col = V3::new(0.0);

	let iters = 50;
	let norm = 1.0 / (iters as f32);
	for i in 0..iters {
		col = col + trace_bounces(&rng.sub(i),o,d) * norm;
	}
	col
}

fn get_color(rng : &Rng, o:V3, d:V3) -> V3 {
	let light_pos = V3(1.0, 1.0, -2.0);
	match trace(o,d, 1000.0) {
		Some(hit) => {
			let light_diff = light_pos-hit.pos;
			let light_dist = light_diff.mag();
			let light_dir = light_diff/light_dist;
			match trace(hit.pos, light_dir, light_dist) {
				Some(_) => V3::new(0.0),
				None => hit.col * light_dir.dot(hit.normal) / (light_dist*light_dist),
			}
		}
		None => sky(d)
	}
}

fn main() {
    let mut file = File::create("out.png").unwrap();

    let width = 800;
    let height = 800;

    let mut frame = Vec::with_capacity(width*height*4);
    frame.resize(width*height*4, 0);

    let cam = V3(0.0,0.0,-2.0);

    let rng = Rng::new(1); // <--- CREATE

    for j in 0..height {
    	let rng = rng.sub(j); // <--- SPLIT

    	for i in 0..width {
    		let rng = rng.sub(i); // <--- SPLIT

    		//calculate camera ray
    		let pos = V3(i as f32, j as f32, 0.0) / V3(width as f32, height as f32, 1.0) * 2.0 - 1.0;
    		let dir = (pos-cam).unit();

    		//trace ray
    		let col = monte_carlo(&rng, pos, dir); // <--- USE

    		//write pixel to image
    		frame[(j*width+i)*4+0] = (col.0 * 256.0).min(255.0).max(0.0) as u8;
    		frame[(j*width+i)*4+1] = (col.1 * 256.0).min(255.0).max(0.0) as u8;
    		frame[(j*width+i)*4+2] = (col.2 * 256.0).min(255.0).max(0.0) as u8;
    		frame[(j*width+i)*4+3] = 255; 
    	}
    }


    png_encode_mini::write_rgba_from_u8(&mut file, &frame[..], width as u32,height as u32).unwrap();
}
