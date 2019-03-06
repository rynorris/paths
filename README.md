paths
-----

*paths* is a software path-tracing renderer written in Rust.

It is purely a hobby-project for me to learn about this technology, and I had no idea how path-tracing worked (other than vague intuition) before starting.

### Implementation Checklist

- [x] Basic path tracing of spheres
- [ ] Point lights
- [x] Multithreading
- [ ] Support 3d models (triangle meshes)
- [ ] Adaptive sampling
- [x] Importance sampling
- [x] Reflective materials
- [ ] Translucent materials (refraction)
- [ ] Bi-directional path tracing
- [ ] Consider direct paths to light sources

### Examples

[2019/03/06] 2 glossy spheres and one perfect mirror sphere on a reflective plane.
![image](https://user-images.githubusercontent.com/3620166/53858421-7ef93000-401d-11e9-9356-31258a0367bd.png)

[2019/03/03] 2 Spheres with large off-screen light source
![image](https://user-images.githubusercontent.com/3620166/53704391-b9b56980-3e5f-11e9-8a36-eb9baaf8630a.png)
