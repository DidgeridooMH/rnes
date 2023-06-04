struct WindowUniform {
  region_aspect: f32,
  window_aspect: f32
};
@group(1) @binding(0)
var<uniform> window: WindowUniform;

struct VertexInput {
  @location(0) position: vec3<f32>
}

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
};

@vertex
fn vs_main(screen_coords: VertexInput) -> VertexOutput {
  var out: VertexOutput;

  var y_scale : f32 = 1.0;
  var x_scale : f32 = 1.0;

  if (window.window_aspect <= window.region_aspect) {
    y_scale = window.window_aspect / window.region_aspect;
  } else {
    x_scale = window.region_aspect / window.window_aspect;
  }

  out.clip_position = vec4<f32>(
      screen_coords.position.x * x_scale,
      screen_coords.position.y *y_scale,
      screen_coords.position.z,
      1.0);
  out.tex_coords = (screen_coords.position.xy + 1.0) * 0.5;
  out.tex_coords.y = (1.0 - out.tex_coords.y);
  return out;
}

@group(0) @binding(0)
  var t_screen: texture_2d<f32>;
@group(0) @binding(1)
  var s_screen: sampler;

  @fragment
  fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_screen, s_screen, in.tex_coords);
  }

