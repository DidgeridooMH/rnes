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
  out.clip_position = vec4<f32>(screen_coords.position, 1.0);
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

