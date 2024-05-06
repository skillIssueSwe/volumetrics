use std::mem::size_of;
use std::sync::Arc;
use noise::{Abs, NoiseFn, Perlin};



pub const top_dim: u32 = 16;
pub const low_dim: u32 = 4;
pub const top_volume: usize = (top_dim * top_dim * top_dim) as usize;
pub const low_volume: usize = (low_dim * low_dim * low_dim) as usize;
pub const brick_data_len: usize = ((low_dim * low_dim * low_dim / 32) + 32) as usize;

struct Superchunk {
    subchunk_indices: [u32; top_volume],
    subchunks: Vec<Subchunk>,
    subregion_bits: [u32; 16]
}

impl Superchunk {
    pub(crate) fn generate() -> Superchunk {
        let mut count = 0;
        let mut count_b =  0;
        let mut subchunks: Vec<Subchunk> = vec![];
        let mut subchunk_indices = [0u32; 4096];
        let mut subregion_bits = [0u32; 16];

        let perlin = noise::Perlin::new(342234);
        let s = (1.0 / 128.0);
        let mut heights = vec![0; 256 * 256];

        for x in 0..(top_dim*low_dim*low_dim) {
            for y in 0..(top_dim * low_dim *low_dim) {
                let height = (perlin.get(
                    [
                        s * (x as f64) - 1.0,
                        s * (y as f64) - 1.0
                    ]
                ) * 256f64).abs() as u32;
                heights[(x + y * top_dim * low_dim * low_dim) as usize] = height;
            }
        }

        for super_x in 0..top_dim {
            for super_y in 0..top_dim {
                for super_z in 0..top_dim {

                    let mut subchunk = Subchunk::new();
                    let mut empty = true;

                    for chunk_x in 0..low_dim {
                        for chunk_y in 0..low_dim {
                            for chunk_z in 0..low_dim {

                                let mut brick = Brick::new();

                                // brick
                                for x in 0..low_dim {
                                    for y in 0..low_dim {

                                        let height_idx = brick_height(super_x, super_y, chunk_x, chunk_y, x, y);
                                        let height = heights[height_idx];

                                        for z in 0..low_dim {
                                            count_b += 1;
                                            if z + (chunk_z * low_dim) + (super_z * low_dim * low_dim) < height as u32 {
                                                count += 1;
                                                empty = false;
                                                let brick_index = (x + y * low_dim + z * low_dim * low_dim) as usize;
                                                let data_index = brick_index / (size_of::<u32>() * 8);
                                                let data_bit_pos = brick_index % (size_of::<u32>() * 8);

                                                brick.data[data_index] |= (1 << data_bit_pos);
                                            }
                                        }
                                    }
                                }
                                if !empty {
                                    subchunk.bricks.push(brick);
                                    subchunk.brick_indices[(chunk_x + chunk_y * low_dim + chunk_z * low_dim * low_dim) as usize] = (((subchunk.bricks.len() - 1) << 24) | 1) as u32;
                                }
                            }
                        }
                    }
                    match empty {
                        false => {
                            subchunks.push(subchunk);
                            subchunk_indices[(super_x + super_y * top_dim + super_z * top_dim * top_dim) as usize] = (((subchunks.len() - 1) << 19) | 1) as u32;

                            if Vec3::from([super_x, super_y, super_z]).is_even() {
                                let subregion_dim = (top_dim / 2);
                                let subregion_index = (super_x + super_y * subregion_dim + super_z * subregion_dim * subregion_dim) as usize;
                                let data_index = subregion_index / (size_of::<u32>() * 8);
                                let data_bit_pos = subregion_index % (size_of::<u32>() * 8);
                                subregion_bits[data_index] |= (1 << data_bit_pos);
                            }
                        }
                        _ => ()
                    }
                }
            }
        }
        println!("[COUNT] {}", count);
        println!("[COUNT_B] {}", count_b);
        Superchunk {
            subchunk_indices,
            subchunks,
            subregion_bits
        }
    }
}

fn generate_sub_chunk(super_x: u32, super_y: u32, super_z: u32) {}

pub fn brick_height(super_x: u32, super_y: u32, chunk_x: u32, chunk_y: u32, x: u32, y: u32) -> usize {
    let nx = x + (chunk_x * low_dim) + (super_x * low_dim * low_dim);
    let ny = y + (chunk_y * low_dim) + (super_y * low_dim * low_dim);
    (nx + ny * low_dim * low_dim * top_dim) as usize
}


fn test_noise() {}

struct Subchunk {
    brick_indices: [u32; low_volume],
    bricks: Vec<Brick>
}

impl Subchunk {
    pub fn new() -> Subchunk {
        Subchunk {
            brick_indices: [0; low_volume],
            bricks: vec![]
        }
    }
}

struct Brick {
    data: [u32; 3],
}

impl Brick {
    pub fn new() -> Brick {
        Brick {
            data: [0; 3]
        }
    }
}

pub struct Vec3 {
    pub x: u32,
    pub y: u32,
    pub z: u32
}

impl Vec3 {
    fn new(x: u32, y: u32, z: u32) -> Self {
        Self {
            x, y, z
        }
    }

    fn from(vec: [u32; 3]) -> Self {
        Self {
            x: vec[0],
            y: vec[1],
            z: vec[2]
        }
    }

    fn is_even(&self) -> bool {
        (self.x % 2 == 0) && (self.y % 2 == 0) && (self.z % 2 == 0)
    }
}
