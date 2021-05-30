#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
// #![deny(warnings)]

use interface::BoardSize;
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use glam::{uvec2, vec3, vec4, UVec2, UVec3, Vec4};

#[repr(u8)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    UR,
    DR,
    DL,
    UL,
}
use Direction::*;

fn position_add(position: UVec2, size: BoardSize, direction: Direction) -> (bool, UVec2) {
    let x = match direction {
        Up | Down => Some(position.x),
        Left | DL | UL => (position.x > 0).then(|| position.x - 1),
        Right | UR | DR => (position.x + 1 < size.width).then(|| position.x + 1),
    };
    let y = match direction {
        Left | Right => Some(position.x),
        Up | UL | UR => (position.y > 0).then(|| position.y - 1),
        Down | DL | DR => (position.y + 1 < size.height).then(|| position.y + 1),
    };
    // TODO: Use Option<UVec2> when that is possible
    match (x, y) {
        (Some(x), Some(y)) => (true, uvec2(x, y)),
        _ => (false, position),
    }
}

fn index_of_pos(position: UVec2, size: BoardSize) -> usize {
    (position.x + position.y * size.width) as usize
}

fn input_at(input: &[u8], size: BoardSize, position: UVec2, direction: Direction) -> u8 {
    let (valid, pos) = position_add(position, size, direction);
    if valid {
        input[index_of_pos(pos, size)]
    } else {
        0
    }
}

fn compute_life_tile(input: &[u8], size: BoardSize, position: UVec2) -> bool {
    let at = move |direction| input_at(input, size, position, direction);
    let total = at(Up) + at(Down) + at(Left) + at(Right) + at(UR) + at(DR) + at(DL) + at(UL);
    let alive = input[index_of_pos(position, size)] == 1;

    alive && (total == 2 || total == 3) || !alive && total == 3
}

#[spirv(compute(threads(32, 32, 1)))]
pub fn life_step(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] board_size: &BoardSize,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] input_data: &[u8],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] output_data: &mut [u8],
) {
    let pos = id.truncate();
    let size = BoardSize {
        width: board_size.width,
        height: board_size.height,
    };
    output_data[index_of_pos(pos, size)] = compute_life_tile(input_data, size, id.truncate()) as u8;
}

#[spirv(vertex)]
pub fn main_vs_colour(
    #[spirv(vertex_index)] in_vertex_index: u32,
    #[spirv(position)] clip_position: &mut Vec4,
    output_color: &mut Vec4,
) {
    let x = (1 - in_vertex_index as i32) as f32 * 0.5;
    let y = ((in_vertex_index & 1) as i32 * 2 - 1) as f32 * 0.5;
    let pos = vec4(x, y, 0., 1.);
    *clip_position = pos;
    let pos = pos + Vec4::X + Vec4::Y;
    let out = vec3(pos.x, pos.y, pos.z) + vec3(pos.z, pos.x, pos.z);
    *output_color = (out.exp() - out.exp().floor()).extend(0.);
}

#[spirv(fragment)]
pub fn main_fs_colour(#[spirv(position)] _in_position: Vec4, input: Vec4, output: &mut Vec4) {
    *output = input;
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] in_vertex_index: u32,
    #[spirv(position)] clip_position: &mut Vec4,
) {
    let x = (1 - in_vertex_index as i32) as f32 * 0.5;
    let y = ((in_vertex_index & 1) as i32 * 2 - 1) as f32 * 0.5;
    *clip_position = vec4(x, y, 0., 1.);
}

#[spirv(fragment)]
pub fn main_fs(#[spirv(position)] _in_position: Vec4, output: &mut Vec4) {
    *output = vec4(0.3, 0.2, 0.1, 1.);
}
