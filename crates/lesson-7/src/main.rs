mod files;
use std::{time::Instant, thread, sync::{Mutex, RwLock, Arc}};
use cgmath::{Vector3, InnerSpace};
use rand::{Rng, prelude::ThreadRng};

use crate::files::write_image_file;

const HEIGHT: usize = 2048;
const WIDTH: usize = HEIGHT;
const BYTES_PER_PIXEL: usize = 3;
const ARRAY_LENGTH: usize = HEIGHT * WIDTH * BYTES_PER_PIXEL;

struct Intersection {
    distance: f32,
    normal: Vector3<f32>,
}

#[derive(Clone, Copy)]
struct Material {
    emittance: Vector3<f32>,
    reflectance: f32,
    specular: f32,
}

struct Sphere {
    center: Vector3<f32>,
    radius: f32,
    material: Material,
}

impl Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        // geometric solution
        let l = self.center - ray.origin; 
        let tca = l.dot(ray.direction); 
        if tca < 0.0 {
            return None;
        }

        let d2 = l.dot(l) - tca * tca; 
        let radius2 = self.radius * self.radius;
        if d2 > radius2 {
            return None; 
        }

        let thc = (radius2 - d2).sqrt(); 
        let t0 = tca - thc; 
        let t1 = tca + thc; 

        let mut t = t0.min(t1);
        if t < 0.0 {
            t = t0.max(t1);
            if t < 0.0 {
                return None;
            }
        }

        let distance = t;
        let position = ray.origin + ray.direction * t;
        let normal = (position - self.center).normalize();
        
        return Some(Intersection {
            distance,
            normal
        });
    }
}

struct DirectionalLight {
    direction: Vector3<f32>,
    intensity: f32,
    color: Vector3<f32>,
}

struct Scene {
    spheres: Vec<Sphere>,
    lights: Vec<DirectionalLight>,
}

#[derive(Clone, Copy)]
struct Camera {
    position: Vector3<f32>,
    forward: Vector3<f32>
}

struct Ray {
    origin: Vector3<f32>,
    direction: Vector3<f32>
}

fn pixel_to_position(camera: &Camera, row: usize, col: usize, row_size: usize, col_size: usize) -> Vector3<f32> {
    let dist_to_screen = 2.0;

    let x = (col as f32 - col_size as f32 / 2.0) / (0.5 * col_size as f32);
    let y = (row as f32 - row_size as f32 / 2.0) / (0.5 * row_size as f32);
    let z = 0.0;
    let point_in_screen_plane = Vector3::new(x, y, z);

    camera.position + camera.forward * dist_to_screen + point_in_screen_plane
}

fn get_color(rng: &mut ThreadRng, scene: &Scene, ray: &Ray, depth: u8, max_depth: u8) -> Vector3<f32> {
    let mut color: Vector3<f32> = Vector3::new(0.,0.,0.);
    if depth == max_depth {
        return color;
    }
    
    let mut min_depth = f32::MAX;
    let mut nearest_intersection = None;
    let mut material = None;
    for sphere in &scene.spheres {
        if let Some(intersection) = sphere.intersect(ray) {
            if intersection.distance >= min_depth {
                continue;
            }
            min_depth = intersection.distance;
            nearest_intersection = Some(intersection);
            material = Some(sphere.material);
        }
    }

    if let Some(intersection) = nearest_intersection {
        let material = material.unwrap();
        const PI: f32 = std::f32::consts::PI;
        const P: f32 = 1. / (2. * PI);

        // fn chi_GGX(v: f32) -> f32 {
        //     return if v > 0.0 { 1.0 } else { 0.0 };
        // }
        
        // fn GGX_distribution(n: Vector3<f32>, h: Vector3<f32>, alpha: f32) -> f32 {
        //     let NoH = n.dot(h);
        //     let alpha2 = alpha * alpha;
        //     let NoH2 = NoH * NoH;
        //     let den = NoH2 * alpha2 + (1.0 - NoH2);
        //     return (chi_GGX(NoH) * alpha2) / ( PI * den * den );
        // }

        // fn GGX_partial_geom_term(v: Vector3<f32>, n: Vector3<f32>, h: Vector3<f32>, alpha: f32) -> f32 {
        //     let VoH2 = v.dot(h).clamp(0.0, 1.0);
        //     let chi = chi_GGX( VoH2 / v.dot(n).clamp(0.0, 1.1));
        //     let VoH2 = VoH2 * VoH2;
        //     let tan2 = ( 1.0 - VoH2 ) / VoH2;
        //     return (chi * 2.0) / ( 1.0 + ( 1.0 + alpha * alpha * tan2 ).sqrt() );
        // }

        // fn fresnel_schlick(cosT: f32, F0: Vector3<f32>) -> Vector3<f32> {
        //     return F0 + (Vector3::new(1.0, 1.0, 1.0) - F0) * (1.0 - cosT).powi(5);
        // }

        // fn fr() {

        // }

        let specular_bounce = rng.gen_bool(material.specular as f64);
        let new_direction = if specular_bounce {
            ray.direction - 2.0 * (ray.direction.dot(intersection.normal)) * intersection.normal
        } else {
            loop {
                let u1: f32 = rng.gen_range(0.0..1.0);
                let u2: f32 = rng.gen_range(0.0..1.0);
                let lambda = f32::acos(2.0*u1 - 1.0) - PI / 2.0;
                let phi = 2.0 * PI * u2;
                let rand_direction = Vector3::new(lambda.cos()*phi.cos(), lambda.cos()*phi.sin(), lambda.sin());

                if rand_direction.dot(intersection.normal) >= 0.0 {
                    break rand_direction;
                }

            }
        };

        let new_origin = ray.origin + ray.direction * intersection.distance;
        let new_ray = Ray {
            origin: new_origin,
            direction: new_direction,
        };

        let cos_theta = new_ray.direction.dot(intersection.normal);
        let brdf = material.reflectance / PI;

        let incoming_bounce_color = get_color(rng, scene, &new_ray, depth + 1, max_depth);

        color += material.emittance + (brdf * incoming_bounce_color * cos_theta / P);
    } else if depth != 0 {
        // Didn't hit anything so return the light color
        for light in &scene.lights {
            let cos_theta = ray.direction.dot(-light.direction).clamp(0.0, 1.0);
            color += light.color * cos_theta * light.intensity;
        }
    }

    return color;
}

// TODO: https://google.github.io/filament/Filament.html#materialsystem/diffusebrdf
fn main() {
    let camera = Camera {
        position: Vector3::new(0.0, 0.0, -10.0),
        forward: Vector3::new(0.0, 0.0, 1.0),
    };

    const PURPLE: Vector3<f32> = Vector3::new(0.8, 0.1, 0.8);
    const RED: Vector3<f32> = Vector3::new(1.0, 0.1, 0.1);
    const GREEN: Vector3<f32> = Vector3::new(0.1, 1.0, 0.1);
    const WHITE: Vector3<f32> = Vector3::new(0.9, 0.9, 0.9);
    const BLACK: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);

    let purple_material = Material {
        emittance: PURPLE,
        reflectance: 1.0,
        specular: 0.0,
    };
    let red_material = Material {
        emittance: RED,
        reflectance: 1.0,
        specular: 0.0,
    };
    let green_material = Material {
        emittance: GREEN,
        reflectance: 1.0,
        specular: 0.0,
    };
    let white_material = Material {
        emittance: WHITE,
        reflectance: 1.0,
        specular: 0.0,
    };
    let black_material = Material {
        emittance: BLACK,
        reflectance: 1.0,
        specular: 0.0,
    };
    let black_shiny_material = Material {
        emittance: BLACK,
        reflectance: 1.0,
        specular: 1.0,
    };

    let scene = Scene {
        lights: vec![
            DirectionalLight { direction: Vector3::new(-1.0, 0.0, 0.0), intensity: 0.5, color: Vector3::new(1.,1.,1.) },
            DirectionalLight { direction: Vector3::new(0.0, -1.0, 0.0), intensity: 0.25, color: Vector3::new(0.0,0.0,1.) },
        ],
        spheres: vec![
            // Sphere {
            //     center: Vector3::new(0.0, -1_000_010.0, 0.0),
            //     radius: 1_000_000.0,
            //     material: black_material,
            // },
            Sphere {
                center: Vector3::new(-4.5, -4.5, 10.0),
                radius: 2.0,
                material: black_shiny_material,
            },
            Sphere {
                center: Vector3::new(-4.5, 4.5, 10.0),
                radius: 2.0,
                material: green_material,
            },
            Sphere {
                center: Vector3::new(4.0, 0.0, 4.5),
                radius: 2.0,
                material: black_material,
            },
            Sphere {
                center: Vector3::new(-1.0, 0.0, 5.0),
                radius: 1.0,
                material: red_material,
            },
            Sphere {
                center: Vector3::new(-3.0, 0.0, 6.0),
                radius: 1.0,
                material: black_material, 
            },
            Sphere {
                center: Vector3::new(-5.0, 0.0, 7.0),
                radius: 1.0,
                material: black_material,
            },
            Sphere {
                center: Vector3::new(0.0, 3.0, 15.0),
                radius: 1.0,
                material: white_material,
            },
            Sphere {
                center: Vector3::new(0.0, 5.0, 16.0),
                radius: 1.0,
                material: black_material,
            },
            Sphere {
                center: Vector3::new(0.0, -2.0, 3.0),
                radius: 1.0,
                material: purple_material,
            },
            Sphere {
                center: Vector3::new(0.3, 0.3, 4.5),
                radius: 1.0,
                material: black_material,
            },
        ]
    };

    let scene = Arc::new(RwLock::new(scene));

    let image_bytes: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::with_capacity(ARRAY_LENGTH)));
    unsafe {
        image_bytes.lock().unwrap().set_len(ARRAY_LENGTH);
    }

    let now = Instant::now();
    let mut results = Vec::new();
    for row in 0..HEIGHT {
        let scene = scene.clone();
        let image_bytes = image_bytes.clone();
        results.push(thread::spawn(move || {
            let scene = scene.read().unwrap();
            let mut rng = rand::thread_rng();
            for col in 0..WIDTH {
                let pixel_pointer: usize = WIDTH * BYTES_PER_PIXEL * row + BYTES_PER_PIXEL * col;
                let blue_pointer = pixel_pointer;
                let green_pointer = pixel_pointer + 1;
                let red_pointer = pixel_pointer + 2;

                let pixel_position = pixel_to_position(&camera, row, col, WIDTH, HEIGHT);

                let ray = Ray {
                    origin: pixel_position,
                    direction: (pixel_position - camera.position).normalize(),
                };

                const SAMPLES: u32 = 32;
                let mut color = Vector3::new(0.0,0.0,0.0);
                for _ in  0..SAMPLES {
                    color += get_color(&mut rng, &scene, &ray, 0, 3);
                }
                color /= SAMPLES as f32;

                let mut image_bytes = image_bytes.lock().unwrap();
                image_bytes[blue_pointer] = (color.z * 255.0).round() as u8;
                image_bytes[green_pointer] = (color.y * 255.0).round() as u8;
                image_bytes[red_pointer] = (color.x * 255.0).round() as u8;
            }
        }));
    }
    for result in results {
        result.join().unwrap();
    }
    println!("{} ms", now.elapsed().as_millis());
    {
        let image_bytes = image_bytes.lock().unwrap();
        let image_bytes = image_bytes.clone();
        write_image_file(WIDTH, HEIGHT, image_bytes);
    }
}