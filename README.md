paths
-----

*paths* is a software path-tracing renderer written in Rust.

It is purely a hobby-project for me to learn about this technology, and I had no idea how path-tracing worked (other than vague intuition) before starting.

### Implementation Checklist

- [x] Basic path tracing of spheres
- [ ] Point lights
- [x] Multithreading
- [x] Collision acceleration (BVH)
- [ ] Textures
- [x] Triangle meshes
- [x] Normal smoothing (of triangle meshes)
- [x] Importance sampling
- [x] Reflective materials
- [x] Glossy materials
- [x] Camera lens simulation
- [ ] Translucent materials (refraction)
- [ ] Subsurface scattering
- [ ] Direct lighting
- [ ] Bi-directional path tracing

### Examples

[2019/03/14] Teapot model with ~6000 triangles and smoothed normals
![image](https://user-images.githubusercontent.com/3620166/54364159-a603d180-46af-11e9-973c-cbab9fac9685.png)

[2019/03/11] 500 randomly generated spheres
![image](https://user-images.githubusercontent.com/3620166/54086894-1b5e6200-4391-11e9-8400-041ce5de0579.png)

[2019/03/07] Close up of green sphere with strong bokeh on the background
![image](https://user-images.githubusercontent.com/3620166/53971014-579a8400-413f-11e9-9bf7-3c5932cb6df1.png)

[2019/03/05] 2 glossy spheres and one perfect mirror sphere on a reflective plane.
![image](https://user-images.githubusercontent.com/3620166/53858421-7ef93000-401d-11e9-9356-31258a0367bd.png)

[2019/03/03] 2 Spheres with large off-screen light source
![image](https://user-images.githubusercontent.com/3620166/53704391-b9b56980-3e5f-11e9-8a36-eb9baaf8630a.png)
