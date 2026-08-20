// Packed cell flags. Keep in sync with `FLAG_*` / `CURSOR_SHIFT` in renderer.rs.
// bit0 underline, bit1 strike, bit2 spacer, bit3 wide, bit4 has_glyph,
// bits5-7 underline kind (1 single, 2 double, 3 curly, 4 dotted, 5 dashed),
// bits8-11 cursor (1 block, 2 bar, 3 underline, 4 hollow),
// bit12 color glyph.

struct Grid {
    cols: u32,
    rows: u32,
    surface_w: f32,
    surface_h: f32,
    cell_w: f32,
    cell_h: f32,
    pad: f32,
    srgb_target: f32,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) bg: vec4<f32>,
    @location(1) fg: vec4<f32>,
    @location(2) atlas_uv: vec2<f32>,
    @location(3) cell_uv: vec2<f32>,
    @location(4) @interpolate(flat) packed: u32,
};

@group(0) @binding(0) var<uniform> grid: Grid;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var atlas_sampler: sampler;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
    @location(0) bg: vec4<f32>,
    @location(1) fg: vec4<f32>,
    @location(2) uv: vec4<f32>,
    @location(3) packed: u32,
) -> VertexOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
    );
    let corner = corners[vertex_index];
    let col = instance_index % grid.cols;
    let row = instance_index / grid.cols;
    let wide = (packed >> 3u) & 1u;
    let span = select(1.0, 2.0, wide == 1u);
    let left = grid.pad + f32(col) * grid.cell_w;
    let top = grid.pad + f32(row) * grid.cell_h;
    let right = left + grid.cell_w * span;
    let bottom = top + grid.cell_h;
    let x = mix(left, right, corner.x);
    let y = mix(top, bottom, corner.y);
    var out: VertexOut;
    out.position = vec4<f32>(
        x * 2.0 / grid.surface_w - 1.0,
        1.0 - y * 2.0 / grid.surface_h,
        0.0,
        1.0,
    );
    out.bg = bg;
    out.fg = fg;
    out.atlas_uv = mix(uv.xy, uv.zw, corner);
    out.cell_uv = corner;
    out.packed = packed;
    return out;
}

fn srgb_to_lin(channel: f32) -> f32 {
    if (channel <= 0.04045) {
        return channel / 12.92;
    }
    return pow((channel + 0.055) / 1.055, 2.4);
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let packed = input.packed;
    // Wide CJK/emoji occupy two instances: the primary (span=2) then a
    // spacer. The spacer must not overwrite the right half of the glyph.
    if (((packed >> 2u) & 1u) == 1u) {
        discard;
    }
    var color = input.bg;
    let has_glyph = ((packed >> 4u) & 1u) == 1u;
    if (has_glyph) {
        let texel = textureSample(atlas, atlas_sampler, input.atlas_uv);
        let color_glyph = ((packed >> 12u) & 1u) == 1u;
        if (color_glyph) {
            var rgb = texel.rgb;
            if (grid.srgb_target > 0.5 && texel.a > 0.001) {
                let straight = rgb / texel.a;
                rgb = vec3<f32>(
                    srgb_to_lin(straight.r),
                    srgb_to_lin(straight.g),
                    srgb_to_lin(straight.b),
                ) * texel.a;
            }
            color = vec4<f32>(color.rgb * (1.0 - texel.a) + rgb, 1.0);
        } else {
            color = mix(color, input.fg, texel.a);
        }
    }
    let strike = ((packed >> 1u) & 1u) == 1u;
    let underline_kind = (packed >> 5u) & 7u;
    let cursor = (packed >> 8u) & 15u;
    let x = input.cell_uv.x;
    let y = input.cell_uv.y;
    if (underline_kind == 1u && y > 0.88) {
        color = input.fg;
    }
    if (underline_kind == 2u && ((y > 0.80 && y < 0.86) || y > 0.92)) {
        color = input.fg;
    }
    if (underline_kind == 3u) {
        let wave = abs(fract(x * 3.0) * 2.0 - 1.0);
        if (abs(y - (0.86 + wave * 0.08)) < 0.045) {
            color = input.fg;
        }
    }
    if (underline_kind == 4u && y > 0.88 && fract(x * 6.0) < 0.45) {
        color = input.fg;
    }
    if (underline_kind == 5u && y > 0.88 && fract(x * 3.5) < 0.72) {
        color = input.fg;
    }
    if (strike && input.cell_uv.y > 0.50 && input.cell_uv.y < 0.58) {
        color = input.fg;
    }
    if (cursor == 2u && input.cell_uv.x < 0.14) {
        color = input.fg;
    }
    if (cursor == 3u && input.cell_uv.y > 0.86) {
        color = input.fg;
    }
    if (cursor == 4u) {
        let edge = input.cell_uv.x < 0.08
            || input.cell_uv.x > 0.92
            || input.cell_uv.y < 0.08
            || input.cell_uv.y > 0.92;
        if (edge) {
            color = input.fg;
        }
    }
    return color;
}
